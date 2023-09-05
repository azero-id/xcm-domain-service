#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod xc_simple_contract {
    use common::{MultiAddress, SimpleContractError};

    #[ink(storage)]
    pub struct XcSimpleContract {
        admin: AccountId,
        primary_handler_contract: MultiAddress,
    }

    impl XcSimpleContract {
        #[ink(constructor)]
        pub fn new(admin: AccountId, primary_handler_contract: MultiAddress) -> Self {
            Self {
                admin,
                primary_handler_contract,
            }
        }

        #[ink(message)]
        pub fn get_value(&self, user: MultiAddress) -> u128 {
            unimplemented!()
        }

        #[ink(message)]
        pub fn set_value(&self, new_value: u128) -> Result<u128, SimpleContractError> {
            unimplemented!()
        }
    }
}
