use super::*;

pub fn verify_contract_state(contract: &AccountId32, id: u128, user: Option<AccountId32>) {
    let get_counter = || {
        let sel_counter = Bytes::from_str("0x94fc951c")
            .map(|v| v.to_vec())
            .expect("unable to parse hex string");

        let data = call_contract(contract, ALICE, sel_counter, 0);
        u128::decode(&mut data.as_bytes_ref()).expect("failed to decode")
    };

    let get_last_visitor = || {
        let sel_last_visitor = Bytes::from_str("0xef8c2bd9")
            .map(|v| v.to_vec())
            .expect("unable to parse hex string");

        let data = call_contract(contract, ALICE, sel_last_visitor, 0);
        Option::<AccountId32>::decode(&mut data.as_bytes_ref()).expect("failed to decode")
    };

    let find_who_am_i = |id: u128| {
        let mut sel_who_am_i = Bytes::from_str("0x8fb9cb05")
            .map(|v| v.to_vec())
            .expect("unable to parse hex string");
        sel_who_am_i.append(&mut id.encode());

        let data = call_contract(contract, ALICE, sel_who_am_i, 0);
        Option::<AccountId32>::decode(&mut data.as_bytes_ref()).expect("failed to decode")
    };

    ParaA::execute_with(|| {
        assert_eq!(get_counter(), id);
        assert_eq!(get_last_visitor(), user);
        assert_eq!(find_who_am_i(id), user);
    });
}

#[test]
fn understand_who_am_i() {
    // super::init_tracing();
    MockNet::reset();

    // 0. Deploy who_am_i contract
    let contract = ParaA::execute_with(|| {
        let blob =
            std::fs::read("./target/ink/who_am_i/who_am_i.wasm").expect("cound not find wasm blob");

        let mut sel_constructor = Bytes::from_str("0x9bae9d5e")
            .map(|v| v.to_vec())
            .expect("unable to parse hex string");
        sel_constructor.append(&mut ALICE.encode());
        sel_constructor.append(&mut ALICE.encode());

        deploy_contract(blob, sel_constructor, ALICE)
    });
    verify_contract_state(&contract, 0, None);

    let sel_walk_in = Bytes::from_str("0xc0397d90")
        .map(|v| v.to_vec())
        .expect("unable to parse hex string");

    // 1. Walk in (ParaA - BOB) (via direct extrinsic)
    ParaA::execute_with(|| {
        assert_ok!(ParachainContracts::call(
            parachain::RuntimeOrigin::signed(BOB),
            contract.clone(),
            0,
            TX_GAS.into(),
            None,
            sel_walk_in.clone(),
        ));
    });
    verify_contract_state(&contract, 1, Some(BOB));

    let call =
        parachain::RuntimeCall::Contracts(pallet_contracts::Call::<parachain::Runtime>::call {
            dest: contract.clone(),
            value: 0,
            gas_limit: TX_GAS.into(),
            storage_deposit_limit: None,
            data: sel_walk_in.clone(),
        });

    let est_wt = parachain::estimate_weight(3) + (3 * TX_GAS).into();
    let fee = parachain::estimate_fee_for_weight(est_wt);

    let message = Xcm(vec![
        WithdrawAsset(vec![(Parent, fee).into()].into()),
        BuyExecution {
            fees: (Parent, fee).into(),
            weight_limit: WeightLimit::Unlimited,
        },
        Transact {
            origin_kind: OriginKind::SovereignAccount,
            require_weight_at_most: (2 * TX_GAS).into(),
            call: call.encode().into(),
        },
    ]);

    // 2. Walk-in (ParaA - Alice) (using execute-xcm)
    ParaA::execute_with(|| {
        assert_ok!(ParachainPalletXcm::execute(
            parachain::RuntimeOrigin::signed(ALICE),
            Box::new(xcm::VersionedXcm::from(message.clone())),
            u64::MAX.into()
        ));
    });
    verify_contract_state(&contract, 2, Some(ALICE));

    // 3. Walk-in (ParaB - ALICE) (using send-xcm)
    ParaB::execute_with(|| {
        assert_ok!(ParachainPalletXcm::send_xcm(
            Junction::AccountId32 {
                id: ALICE.into(),
                network: None,
            },
            (Parent, Parachain(1)),
            message.into()
        ));
    });

    let alice_b_on_a = sibling_account_sovereign_account_id(2, ALICE);
    verify_contract_state(&contract, 3, Some(alice_b_on_a));
}

#[test]
fn test_xc_walk_in() {
    // super::init_tracing();
    MockNet::reset();

    // 0a. Deploy who_am_i contract
    let contract = ParaA::execute_with(|| {
        let blob =
            std::fs::read("./target/ink/who_am_i/who_am_i.wasm").expect("cound not find wasm blob");

        let mut sel_constructor = Bytes::from_str("0x9bae9d5e")
            .map(|v| v.to_vec())
            .expect("unable to parse hex string");
        sel_constructor.append(&mut ALICE.encode());
        sel_constructor.append(&mut ALICE.encode());

        deploy_contract(blob, sel_constructor, ALICE)
    });
    verify_contract_state(&contract, 0, None);

    // 0b. Deploy xc_who_am_i contract
    let xc_contract = ParaB::execute_with(|| {
        let blob = std::fs::read("./target/ink/xc_who_am_i/xc_who_am_i.wasm")
            .expect("cound not find wasm blob");

        let mut sel_constructor = Bytes::from_str("0x9bae9d5e")
            .map(|v| v.to_vec())
            .expect("unable to parse hex string");
        sel_constructor.append(&mut contract.encode());
        sel_constructor.append(&mut ALICE.encode());    // Dummy value; callback not used

        deploy_contract(blob, sel_constructor, ALICE)
    });

    // 1. Fund xc_contract for ParaA gas
    let sovereign_xc_contract_addr = sibling_account_sovereign_account_id(2, xc_contract.clone());

    ParaA::execute_with(|| {
        assert_ok!(ParachainBalances::force_set_balance(
            parachain::RuntimeOrigin::root(),
            sovereign_xc_contract_addr.clone(),
            INITIAL_BALANCE,
        ));

        assert_ok!(ParachainAssets::mint(
            parachain::RuntimeOrigin::signed(ADMIN),
            0,
            sovereign_xc_contract_addr.clone(),
            INITIAL_BALANCE
        ));
    });

    let sel_walk_in = Bytes::from_str("0xc0397d90")
        .map(|v| v.to_vec())
        .expect("unable to parse hex string");

    ParaB::execute_with(|| {
        let data = call_contract(&xc_contract, ALICE, sel_walk_in, 0);
        assert_eq!(data, [0]);
    });

    verify_contract_state(&contract, 1, Some(sovereign_xc_contract_addr));
}
