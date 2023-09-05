#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod handler_simple_contract {
    use common::MultiAddress;
    use common::SimpleContractError;

    #[ink(storage)]
    pub struct HandlerSimpleContract {
        admin: AccountId,
        simple_contract: AccountId,
    }

    impl HandlerSimpleContract {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(admin: AccountId, simple_contract: AccountId) -> Self {
            Self {
                admin,
                simple_contract,
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
