pub use crate::testnet_config::*;
pub use codec::{Decode, Encode};
pub use frame_support::assert_ok;
pub use sp_core::{hexdisplay::AsBytesRef, Bytes};
pub use sp_runtime::AccountId32;
pub use std::str::FromStr;
use std::sync::Once;
pub use xcm::v3::prelude::*;
pub use xcm_simulator::TestExt;

pub const TX_GAS: u64 = 100_000_000_000;

mod who_am_i;

pub fn deploy_contract(blob: Vec<u8>, sel_constr: Vec<u8>, deployer: AccountId32) -> AccountId32 {
    let resp = ParachainContracts::bare_instantiate(
        deployer,
        0,
        TX_GAS.into(),
        None,
        pallet_contracts_primitives::Code::Upload(blob),
        sel_constr,
        vec![],
        true,
    );

    resp.result.expect("Failed to init contract").account_id
}

pub fn call_contract(
    contract: &AccountId32,
    caller: AccountId32,
    msg: Vec<u8>,
    value: Balance,
) -> Vec<u8> {
    let rs = ParachainContracts::bare_call(
        caller,
        contract.clone(),
        value,
        TX_GAS.into(),
        None,
        msg,
        true,
        pallet_contracts::Determinism::Enforced,
    )
    .result
    .expect("execution without result");

    let pallet_contracts_primitives::ExecReturnValue { flags, mut data } = rs;
    // empty flags determines succesful execution
    assert!(flags.is_empty());
    // InkLang error check
    assert_eq!(data.remove(0), 0);

    data
}

static INIT: Once = Once::new();
fn init_tracing() {
    INIT.call_once(|| {
        // Add test tracing (from sp_tracing::init_for_tests()) but filtering for xcm logs only
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .with_env_filter("xcm=trace,system::events=trace") // Comment out this line to see all traces
            .with_test_writer()
            .init();
    });
}
