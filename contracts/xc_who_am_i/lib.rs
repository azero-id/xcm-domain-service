#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod xc_who_am_i {
    use common::xcm_utils::make_xcm_contract_call;
    use xcm::v3 as xcm_v3;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum RuntimeError {
        CallRuntimeFailed,
    }

    impl From<ink::env::Error> for RuntimeError {
        fn from(e: ink::env::Error) -> Self {
            match e {
                ink::env::Error::CallRuntimeFailed => RuntimeError::CallRuntimeFailed,
                _ => panic!("Unexpected error from `pallet-contracts`."),
            }
        }
    }

    #[ink(storage)]
    pub struct XcWhoAmI {
        contract: AccountId,
    }

    impl XcWhoAmI {
        #[ink(constructor)]
        pub fn new(contract: AccountId) -> Self {
            Self { contract }
        }

        #[ink(message)]
        pub fn walk_in(&mut self) -> core::result::Result<(), RuntimeError> {
            let sel_walk_in = ink::selector_bytes!("walk_in");

            make_xcm_contract_call::<Self>(
                (xcm_v3::Parent, xcm_v3::Junction::Parachain(1)).into(),
                self.contract,
                sel_walk_in.to_vec(),
                0,
                None,
            )
            .map_err(Into::into)
        }
    }
}
