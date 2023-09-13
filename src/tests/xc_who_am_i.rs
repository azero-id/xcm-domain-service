use super::*;

fn encode_selector(sel: &str) -> [u8; 4] {
    let bytes = Bytes::from_str(sel).unwrap().0;
    [bytes[0], bytes[1], bytes[2], bytes[3]]
}

fn deploy_state_manager() -> AccountId32 {
    let blob =
        std::fs::read("./target/ink/who_am_i/who_am_i.wasm").expect("cound not find wasm blob");

    let sel_constructor = encode_selector("0x9bae9d5e");
    let payload = (sel_constructor, ALICE, ALICE).encode(); // (selector, admin, handler)

    deploy_contract(blob, payload, ALICE)
}

fn deploy_xcm_handler(state_manager: &AccountId32) -> AccountId32 {
    let blob = std::fs::read("./target/ink/handler_who_am_i/handler_who_am_i.wasm")
        .expect("cound not find wasm blob");

    let sel_constructor = encode_selector("0x9bae9d5e");
    let payload = (sel_constructor, ALICE, state_manager).encode(); // (selector, admin, state_manager)

    deploy_contract(blob, payload, ALICE)
}

fn deploy_xc_contract(xcm_handler: &AccountId32, xcm_handler_soac: &AccountId32) -> AccountId32 {
    let blob = std::fs::read("./target/ink/xc_who_am_i/xc_who_am_i.wasm")
        .expect("cound not find wasm blob");

    let sel_constructor = encode_selector("0x9bae9d5e");
    let payload = (sel_constructor, xcm_handler, xcm_handler_soac).encode(); // (selector, xcm_handler, xcm_handler_sovereign_account)

    deploy_contract(blob, payload, ALICE)
}

fn set_handler(state_manager: &AccountId32, xcm_handler: &AccountId32) {
    let sel_set_handler = encode_selector("0xee45cea1");
    let payload = (sel_set_handler, xcm_handler).encode();

    let encoded_resp = call_contract(state_manager, ALICE, payload, 0);
    let resp: Result<(), u8> = Decode::decode(&mut &encoded_resp[..]).expect("failed to decode");

    assert_eq!(resp, Ok(()));
}

fn add_xc_contract(
    xcm_handler: &AccountId32,
    xc_contract_soac: &AccountId32,
    location: &(u32, AccountId32),
) {
    let sel_add_xc_contract = encode_selector("0x5578fb41");
    let payload = (sel_add_xc_contract, xc_contract_soac, location).encode();

    let encoded_resp = call_contract(xcm_handler, ALICE, payload, 0);
    let resp: Result<(), u8> = Decode::decode(&mut &encoded_resp[..]).expect("failed to decode");

    assert_eq!(resp, Ok(()));
}

fn fund_address(addr: &AccountId32) {
    assert_ok!(ParachainBalances::force_set_balance(
        parachain::RuntimeOrigin::root(),
        addr.clone(),
        INITIAL_BALANCE,
    ));

    assert_ok!(ParachainAssets::mint(
        parachain::RuntimeOrigin::signed(ADMIN),
        0,
        addr.clone(),
        INITIAL_BALANCE
    ));
}

fn setup() -> (AccountId32, AccountId32, AccountId32) {
    // 1. Deploy `who_am_i`
    let state_manager = ParaA::execute_with(deploy_state_manager);
    println!("state_manager: {:?}", state_manager);

    // 2A. Deploy `handler_who_am_i`
    let xcm_handler = ParaA::execute_with(|| deploy_xcm_handler(&state_manager));
    let xcm_handler_soac = sibling_account_sovereign_account_id(1, xcm_handler.clone());
    println!("xcm_handler: {:?}", xcm_handler);

    // 2B. Update state_manager::handler
    ParaA::execute_with(|| set_handler(&state_manager, &xcm_handler));

    // 3A. Deploy `xc_who_am_i`
    let xc_contract = ParaB::execute_with(|| deploy_xc_contract(&xcm_handler, &xcm_handler_soac));
    let xc_contract_soac = sibling_account_sovereign_account_id(2, xc_contract.clone());
    println!("xc_contract: {:?}", xc_contract);

    // 3B. Approve xc_contract on xcm_handler
    let location = (2, xc_contract.clone());
    ParaA::execute_with(|| add_xc_contract(&xcm_handler, &xc_contract_soac, &location));

    // 4. Fund sovereign accounts for gas fee payment
    ParaB::execute_with(|| fund_address(&xcm_handler_soac));
    ParaA::execute_with(|| fund_address(&xc_contract_soac));

    (state_manager, xcm_handler, xc_contract)
}

#[test]
fn setup_works() {
    MockNet::reset();
    setup();
}

#[test]
fn callback_works() {
    MockNet::reset();

    let (state_manager, xcm_handler, xc_contract) = setup();

    // 1. Walk-in (via xc-contract on ParaB)
    ParaB::execute_with(|| {
        let sel_walk_in = encode_selector("0xc0397d90");
        let data = call_contract(&xc_contract, BOB, sel_walk_in.encode(), 0);
        let rs: Result<(), u8> = Decode::decode(&mut &data[..]).expect("failed to decode");

        assert_eq!(rs, Ok(()));
    });

    // Verify data stored in the state_manager
    crate::tests::who_am_i::verify_contract_state(&state_manager, 1, Some(BOB));

    // 2. Request data for `who_am_i(id)` on ParaB
    let id: u128 = 1;
    let tid_0 = ParaB::execute_with(|| {
        let sel_who_am_i = encode_selector("0x8fb9cb05");
        let payload = (sel_who_am_i, id).encode();

        let data = call_contract(&xc_contract, ALICE, payload, 0);
        let rs: Result<u128, u8> = Decode::decode(&mut &data[..]).expect("failed to decode");
        rs.unwrap()
    });
    assert_eq!(tid_0, 0);

    // 3. Retrieve data on ParaB
    let who_am_i = ParaB::execute_with(|| {
        let sel_retrieve_who_am_i = encode_selector("0xafc817a8");
        let payload = (sel_retrieve_who_am_i, tid_0).encode();

        let data = call_contract(&xc_contract, ALICE, payload, 0);
        let rs: Result<Option<AccountId32>, u8> =
            Decode::decode(&mut &data[..]).expect("failed to decode");
        rs
    });
    assert_eq!(who_am_i, Ok(Some(BOB)));
}
