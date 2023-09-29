use super::*;

fn deploy_state_manager(admin: &AccountId32, handler: &AccountId32) -> AccountId32 {
    let blob = std::fs::read("./contracts/target/ink/domain_service/domain_service.wasm")
        .expect("cound not find wasm blob");

    let sel_constructor = get_selector("new");
    let payload = (sel_constructor, admin, handler).encode(); // (selector, admin, handler)

    deploy_contract(blob, payload, ALICE)
}

fn deploy_xcm_handler(admin: &AccountId32, state_manager: &AccountId32) -> AccountId32 {
    let blob = std::fs::read("./contracts/target/ink/xcm_handler/xcm_handler.wasm")
        .expect("cound not find wasm blob");

    let sel_constructor = get_selector("new");
    let payload = (sel_constructor, admin, state_manager).encode(); // (selector, admin, state_manager)

    deploy_contract(blob, payload, ALICE)
}

pub fn deploy_xc_contract(
    xcm_handler: &AccountId32,
    xcm_handler_soac: &AccountId32,
) -> AccountId32 {
    let blob = std::fs::read("./contracts/target/ink/xc_domain_service/xc_domain_service.wasm")
        .expect("cound not find wasm blob");

    let sel_constructor = get_selector("new");
    let payload = (sel_constructor, xcm_handler, xcm_handler_soac).encode(); // (selector, xcm_handler, xcm_handler_soac)

    deploy_contract(blob, payload, ALICE)
}

fn set_handler(state_manager: &AccountId32, xcm_handler: &AccountId32) {
    let sel_set_handler = get_selector("set_handler");
    let payload = (sel_set_handler, xcm_handler).encode(); // (selector, new_handler)

    let encoded_resp = call_contract(state_manager, ALICE, payload, 0);
    let resp: Result<(), u8> = Decode::decode(&mut &encoded_resp[..]).expect("failed to decode");

    assert_eq!(resp, Ok(()));
}

pub fn add_xc_contract(
    xcm_handler: &AccountId32,
    xc_contract_soac: &AccountId32,
    origin_path: &(u8, u32, AccountId32),
) {
    let sel_add_xc_contract = get_selector("add_xc_contract");
    let payload = (sel_add_xc_contract, xc_contract_soac, origin_path).encode(); // (selector, xc_contract_soac, origin_path)

    let encoded_resp = call_contract(xcm_handler, ALICE, payload, 0);
    let resp: Result<(), u8> = Decode::decode(&mut &encoded_resp[..]).expect("failed to decode");

    assert_eq!(resp, Ok(()));
}

pub fn fund_address(addr: &AccountId32) {
    assert_ok!(ParachainBalances::force_set_balance(
        parachain::RuntimeOrigin::root(),
        addr.clone(),
        INITIAL_BALANCE
    ));
}

pub fn setup() -> (AccountId32, AccountId32, AccountId32) {
    // 1. Deploy `domain_service: state-handler`
    let state_manager = ParaA::execute_with(|| deploy_state_manager(&ALICE, &ALICE));
    println!("state_manager: {:?}", state_manager);

    // 2A. Deploy `xcm_handler`
    let xcm_handler = ParaA::execute_with(|| deploy_xcm_handler(&ALICE, &state_manager));
    let xcm_handler_soac = sibling_account_account_id(1, xcm_handler.clone());
    println!("xcm_handler: {:?}", xcm_handler);

    // 2B. Update state_manager::set_handler
    ParaA::execute_with(|| set_handler(&state_manager, &xcm_handler));

    // 3A. Deploy `xc_domain_service: xc-contract`
    let xc_contract = ParaB::execute_with(|| deploy_xc_contract(&xcm_handler, &xcm_handler_soac));
    let xc_contract_soac = sibling_account_account_id(2, xc_contract.clone());
    println!("xc_contract: {:?}", xc_contract);

    // // 3B. Approve xc_contract on xcm_handler
    let origin_path = (1, 2, xc_contract.clone());
    ParaA::execute_with(|| add_xc_contract(&xcm_handler, &xc_contract_soac, &origin_path));

    // 4. Fund sovereign accounts for gas fee payment
    ParaA::execute_with(|| fund_address(&xc_contract_soac));
    ParaB::execute_with(|| fund_address(&xcm_handler_soac));

    (state_manager, xcm_handler, xc_contract)
}

#[test]
fn setup_works() {
    MockNet::reset();
    setup();
}
