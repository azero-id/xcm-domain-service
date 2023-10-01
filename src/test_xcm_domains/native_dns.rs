use super::*;

pub fn register_name(
    state_manager: &AccountId32,
    caller: AccountId32,
    name: &str,
) -> Result<(), u8> {
    let sel_register_name = get_selector("register_name");
    let payload = (sel_register_name, name).encode();

    let encoded_resp = call_contract(&state_manager, caller, payload, 100);
    Decode::decode(&mut &encoded_resp[..]).expect("failed to decode")
}

pub fn transfer_name(
    state_manager: &AccountId32,
    caller: AccountId32,
    name: &str,
    receiver: AccountId32,
) -> Result<(), u8> {
    let sel_transfer_name = get_selector("transfer_name");
    let payload = (sel_transfer_name, name, receiver).encode();

    let encoded_resp = call_contract(&state_manager, caller, payload, 0);
    Decode::decode(&mut &encoded_resp[..]).expect("failed to decode")
}

pub fn set_address(
    state_manager: &AccountId32,
    caller: AccountId32,
    name: &str,
    address: &(u8, Option<u32>, AccountId32),
) -> Result<(), u8> {
    let sel_set_address = get_selector("set_address");
    let payload = (sel_set_address, name, address).encode();

    let encoded_resp = call_contract(&state_manager, caller, payload, 0);
    Decode::decode(&mut &encoded_resp[..]).expect("failed to decode")
}

pub fn get_owner(state_manager: &AccountId32, name: &str) -> Option<AccountId32> {
    let sel_get_owner = get_selector("get_owner");
    let payload = (sel_get_owner, name).encode();

    let encoded_resp = call_contract(&state_manager, ALICE, payload, 0);
    Decode::decode(&mut &encoded_resp[..]).expect("failed to decode")
}

pub fn get_address(state_manager: &AccountId32, name: &str) -> Option<VersionedMultiLocation> {
    let sel_get_address = get_selector("get_address");
    let payload = (sel_get_address, name).encode();

    let encoded_resp = call_contract(&state_manager, ALICE, payload, 0);
    Decode::decode(&mut &encoded_resp[..]).expect("failed to decode")
}

#[test]
fn registration_works() {
    MockNet::reset();
    let (state_manager, _, _) = setup::setup();

    ParaA::execute_with(|| {
        // Register a name
        let rs = register_name(&state_manager, ALICE, "alice");
        assert_eq!(rs, Ok(()));

        // Get domain's owner
        let rs = get_owner(&state_manager, "alice");
        assert_eq!(rs, Some(ALICE));

        // Get domain's resolving address
        let rs = get_address(&state_manager, "alice");
        assert_eq!(rs, None);
    });
}

#[test]
fn set_address_works() {
    MockNet::reset();
    let (state_manager, _, _) = setup::setup();

    ParaA::execute_with(|| {
        // Register a name
        register_name(&state_manager, ALICE, "alice").unwrap();

        // Set domain's resolving MultiLocation address
        let address = (0, None, ALICE);
        let rs = set_address(&state_manager, ALICE, "alice", &address);
        assert_eq!(rs, Ok(()));

        // Get domain's resolving address
        let true_ml = Junction::AccountId32 {
            network: None,
            id: ALICE.into(),
        };
        let loc = VersionedMultiLocation::V3(true_ml.into());

        let rs = get_address(&state_manager, "alice");
        assert_eq!(rs, Some(loc));
    });
}

#[test]
fn transfer_name_works() {
    MockNet::reset();
    let (state_manager, _, _) = setup::setup();

    ParaA::execute_with(|| {
        // Register a name
        register_name(&state_manager, ALICE, "alice").unwrap();

        // Transfer the name
        let rs = transfer_name(&state_manager, ALICE, "alice", BOB);
        assert_eq!(rs, Ok(()));

        // Verify owner is updated
        let rs = get_owner(&state_manager, "alice");
        assert_eq!(rs, Some(BOB));
    });
}
