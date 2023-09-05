#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod simple_contract {
    use common::MultiAddress;
    use ink::storage::Mapping;

    use common::SimpleContractError;

    #[ink(storage)]
    pub struct SimpleContract {
        admin: AccountId,
        handler: AccountId,
        value: Mapping<MultiAddress, u128>,
    }

    impl SimpleContract {
        #[ink(constructor)]
        pub fn new(admin: AccountId, handler: AccountId) -> Self {
            Self {
                admin,
                handler,
                value: Mapping::default(),
            }
        }

        #[ink(message)]
        pub fn get_value(&self, user: MultiAddress) -> u128 {
            self.value.get(user).unwrap_or(0)
        }

        #[ink(message)]
        pub fn get_admin(&self) -> AccountId {
            self.admin
        }

        #[ink(message)]
        pub fn get_handler(&self) -> AccountId {
            self.handler
        }

        #[ink(message)]
        pub fn set_value(
            &mut self,
            user: MultiAddress,
            new_value: u128,
        ) -> Result<u128, SimpleContractError> {
            self.ensure_handler()?;
            let old_value = self.get_value(user);
            self.value.insert(user, &new_value);
            Ok(old_value)
        }

        #[ink(message)]
        pub fn set_admin(&mut self, new_admin: AccountId) -> Result<(), SimpleContractError> {
            self.ensure_admin()?;
            self.admin = new_admin;
            Ok(())
        }

        #[ink(message)]
        pub fn set_handler(&mut self, new_handler: AccountId) -> Result<(), SimpleContractError> {
            self.ensure_admin()?;
            self.handler = new_handler;
            Ok(())
        }

        fn ensure_admin(&self) -> Result<(), SimpleContractError> {
            if self.env().caller() != self.admin {
                Err(SimpleContractError::NotAdmin)?;
            }
            Ok(())
        }

        fn ensure_handler(&self) -> Result<(), SimpleContractError> {
            if self.env().caller() != self.handler {
                Err(SimpleContractError::NotHandler)?;
            }
            Ok(())
        }
    }
}
