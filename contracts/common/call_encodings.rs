use ink::prelude::boxed::Box;
use ink::prelude::vec::Vec;
use ink::primitives::AccountId;

// @dev Make sure indexes are valid for chain in use!
#[derive(scale::Encode)]
pub enum RuntimeCall {
    #[codec(index = 70)]
    Contracts(ContractsCall),
    #[codec(index = 99)]
    Xcm(XcmCall),
}

#[derive(scale::Encode)]
pub enum XcmCall {
    #[codec(index = 0)]
    Send {
        dest: Box<xcm::VersionedMultiLocation>,
        message: Box<xcm::VersionedXcm<()>>,
    },
}

#[derive(scale::Encode)]
pub enum ContractsCall {
    #[codec(index = 6)]
    Call {
        dest: AccountId,
        #[codec(compact)]
        value: u128,
        gas_limit: crate::Weight,
        storage_deposit_limit: Option<u128>,
        data: Vec<u8>,
    },
}
