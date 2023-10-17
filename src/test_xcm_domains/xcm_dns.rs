use super::*;

pub type TicketId = u128;

pub fn register_name(xc_contract: &AccountId32, caller: AccountId32, name: &str) -> Result<(), u8> {
    let sel_register_name = get_selector("register_name");
    let payload = (sel_register_name, name).encode();

    let encoded_resp = call_contract(&xc_contract, caller, payload, 100);
    Decode::decode(&mut &encoded_resp[..]).expect("failed to decode")
}

pub fn transfer_name(
    xc_contract: &AccountId32,
    caller: AccountId32,
    name: &str,
    receiver: AccountId32,
) -> Result<(), u8> {
    let sel_transfer_name = get_selector("transfer_name");
    let payload = (sel_transfer_name, name, receiver).encode();

    let encoded_resp = call_contract(&xc_contract, caller, payload, 0);
    Decode::decode(&mut &encoded_resp[..]).expect("failed to decode")
}

pub fn set_address(
    xc_contract: &AccountId32,
    caller: AccountId32,
    name: &str,
    address: &(u8, Option<u32>, AccountId32),
) -> Result<(), u8> {
    let sel_set_address = get_selector("set_address");
    let payload = (sel_set_address, name, address).encode();

    let encoded_resp = call_contract(&xc_contract, caller, payload, 0);
    Decode::decode(&mut &encoded_resp[..]).expect("failed to decode")
}

pub fn get_owner(xc_contract: &AccountId32, name: &str) -> Result<TicketId, u8> {
    let sel_get_owner = get_selector("get_owner");
    let payload = (sel_get_owner, name).encode();

    let encoded_resp = call_contract(&xc_contract, ALICE, payload, 0);
    Decode::decode(&mut &encoded_resp[..]).expect("failed to decode")
}

pub fn get_address(xc_contract: &AccountId32, name: &str) -> Result<TicketId, u8> {
    let sel_get_address = get_selector("get_address");
    let payload = (sel_get_address, name).encode();

    let encoded_resp = call_contract(&xc_contract, ALICE, payload, 0);
    Decode::decode(&mut &encoded_resp[..]).expect("failed to decode")
}

pub fn retrieve_owner(xc_contract: &AccountId32, tid: TicketId) -> Result<Option<AccountId32>, u8> {
    let sel_retrieve_owner = get_selector("retrieve_owner");
    let payload = (sel_retrieve_owner, tid).encode();

    let encoded_resp = call_contract(&xc_contract, ALICE, payload, 0);
    Decode::decode(&mut &encoded_resp[..]).expect("failed to decode")
}

pub fn retrieve_address(
    xc_contract: &AccountId32,
    tid: TicketId,
) -> Result<Option<VersionedMultiLocation>, u8> {
    let sel_retrieve_address = get_selector("retrieve_address");
    let payload = (sel_retrieve_address, tid).encode();

    let encoded_resp = call_contract(&xc_contract, ALICE, payload, 0);
    Decode::decode(&mut &encoded_resp[..]).expect("failed to decode")
}

#[test]
fn registration_works() {
    MockNet::reset();
    let (state_manager, _, xc_contract) = setup::setup();

    ParaB::execute_with(|| {
        // Register a name
        let rs = register_name(&xc_contract, ALICE, "alice");
        assert_eq!(rs, Ok(()));

        // Request for owner details
        let rs = get_owner(&xc_contract, "alice");
        assert_eq!(rs, Ok(0)); // tid = 0
    });

    ParaB::execute_with(|| {
        // Retrieve owner details from request:0
        let rs = retrieve_owner(&xc_contract, 0);
        assert_eq!(rs, Ok(Some(ALICE)));
    });

    // Verify the state is updated on ParaA::state_manager as well
    ParaA::execute_with(|| {
        let rs = native_dns::get_owner(&state_manager, "alice");
        assert_eq!(rs, Some(ALICE));
    });
}

#[test]
fn set_address_works() {
    MockNet::reset();
    let (state_manager, _, xc_contract) = setup::setup();

    ParaB::execute_with(|| {
        // Register a name
        let rs = register_name(&xc_contract, ALICE, "alice");
        assert_eq!(rs, Ok(()));

        // Set domain's resolving address
        let address = (0, None, ALICE);
        let rs = set_address(&xc_contract, ALICE, "alice", &address);
        assert_eq!(rs, Ok(()));

        // Request for domain's resolving address
        let rs = get_address(&xc_contract, "alice");
        assert_eq!(rs, Ok(0)); // tid = 0
    });

    ParaB::execute_with(|| {
        let true_ml = Junction::AccountId32 {
            network: None,
            id: ALICE.into(),
        };
        let loc = VersionedMultiLocation::V3(true_ml.into());

        // Retrieve resolving address from request:0
        let rs = retrieve_address(&xc_contract, 0);
        assert_eq!(rs, Ok(Some(loc)));
    });

    // Verify the state is updated on ParaA::state_manager as well
    // and the MultiLocation anchoring is working properly
    ParaA::execute_with(|| {
        let account = Junction::AccountId32 {
            network: None,
            id: ALICE.into(),
        };
        let loc = VersionedMultiLocation::V3((Parent, Parachain(2), account).into());

        let rs = native_dns::get_address(&state_manager, "alice");
        assert_eq!(rs, Some(loc));
    });
}

#[test]
fn transfer_works() {
    MockNet::reset();
    let (state_manager, _, xc_contract) = setup::setup();

    ParaB::execute_with(|| {
        // Register a name
        register_name(&xc_contract, ALICE, "alice").unwrap();

        // Transfer the name
        let rs = transfer_name(&xc_contract, ALICE, "alice", BOB);
        assert_eq!(rs, Ok(()));

        // Request for owner details
        let rs = get_owner(&xc_contract, "alice");
        assert_eq!(rs, Ok(0)); // tid = 0
    });

    ParaB::execute_with(|| {
        // Retrieve owner details from request:0
        let rs = retrieve_owner(&xc_contract, 0);
        assert_eq!(rs, Ok(Some(BOB)));
    });

    // Verify the state is updated on ParaA::state_manager as well
    ParaA::execute_with(|| {
        let rs = native_dns::get_owner(&state_manager, "alice");
        assert_eq!(rs, Some(BOB));
    });
}
