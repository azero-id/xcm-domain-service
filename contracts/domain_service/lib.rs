#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod domain_service {
    use ink::prelude::string::String;
    use ink::storage::Mapping;
    use utils::MultilocationEncoded;

    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotAdmin,
        NotHandler,
        NotOwner,
        NameAlreadyExists,
        NameDoesNotExists,
        PaymentNotReceived,
    }

    #[ink(storage)]
    pub struct DomainService {
        admin: AccountId,
        handler: AccountId,
        name_to_owner: Mapping<String, AccountId>,
        name_to_multilocation: Mapping<String, MultilocationEncoded>,
    }

    impl DomainService {
        #[ink(constructor)]
        pub fn new(admin: AccountId, handler: AccountId) -> Self {
            Self {
                admin,
                handler,
                name_to_owner: Mapping::default(),
                name_to_multilocation: Mapping::default(),
            }
        }

        /** Getters STARTS here */

        #[ink(message)]
        pub fn get_admin(&self) -> AccountId {
            self.admin
        }

        /// Returns the xcm-handler address
        #[ink(message)]
        pub fn get_handler(&self) -> AccountId {
            self.handler
        }

        #[ink(message)]
        pub fn get_owner(&self, name: String) -> Option<AccountId> {
            self.name_to_owner.get(name)
        }

        #[ink(message)]
        pub fn get_address(&self, name: String) -> Option<xcm::VersionedMultiLocation> {
            self.name_to_multilocation.get(name).map(Into::into)
        }

        /** Getters ENDS here */

        /** Setters for NATIVE calls STARTS here */

        #[ink(message, payable)]
        pub fn register_name(&mut self, name: String) -> Result<(), Error> {
            let caller = self.env().caller();
            if self.env().transferred_value() < 100 {
                return Err(Error::PaymentNotReceived);
            }
            self.do_register_name(&caller, &name)
        }

        #[ink(message)]
        pub fn transfer_name(&mut self, name: String, receiver: AccountId) -> Result<(), Error> {
            let caller = self.env().caller();
            self.do_transfer_name(&caller, &name, &receiver)
        }

        #[ink(message)]
        pub fn set_address(
            &mut self,
            name: String,
            loc: MultilocationEncoded,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            self.do_set_address(&caller, &name, &loc)
        }

        /** Setters for NATIVE calls ENDS here */

        /** Setters for XCM calls STARTS here */

        /// For xcm-handler use only!
        #[ink(message)]
        pub fn xcm_register_name(&mut self, caller: AccountId, name: String) -> Result<(), Error> {
            self.ensure_handler()?;
            self.do_register_name(&caller, &name)
        }

        /// For xcm-handler use only!
        #[ink(message)]
        pub fn xcm_transfer_name(
            &mut self,
            caller: AccountId,
            name: String,
            receiver: AccountId,
        ) -> Result<(), Error> {
            self.ensure_handler()?;
            self.do_transfer_name(&caller, &name, &receiver)
        }

        /// For xcm-handler use only!
        #[ink(message)]
        pub fn xcm_set_address(
            &mut self,
            caller: AccountId,
            name: String,
            loc: MultilocationEncoded,
        ) -> Result<(), Error> {
            self.ensure_handler()?;
            self.do_set_address(&caller, &name, &loc)
        }

        /** Setters for XCM calls ENDS here */

        /** Privileged messages STARTS here */

        /// ADMIN use only! Allows updating the xcm-handler address
        #[ink(message)]
        pub fn set_handler(&mut self, new_handler: AccountId) -> Result<(), Error> {
            self.ensure_admin()?;
            self.handler = new_handler;
            Ok(())
        }

        /** Privileged messages ENDS here */

        fn do_register_name(&mut self, caller: &AccountId, name: &str) -> Result<(), Error> {
            if self.name_to_owner.contains(name) {
                return Err(Error::NameAlreadyExists);
            }
            self.name_to_owner.insert(name, caller);
            Ok(())
        }

        fn do_transfer_name(
            &mut self,
            caller: &AccountId,
            name: &str,
            receiver: &AccountId,
        ) -> Result<(), Error> {
            let Some(owner) = self.name_to_owner.get(name) else {
                return Err(Error::NameDoesNotExists);
            };
            if caller != &owner {
                return Err(Error::NotOwner);
            }

            self.name_to_owner.insert(name, receiver);
            self.name_to_multilocation.remove(name);
            Ok(())
        }

        fn do_set_address(
            &mut self,
            caller: &AccountId,
            name: &str,
            loc: &MultilocationEncoded,
        ) -> Result<(), Error> {
            let Some(owner) = self.name_to_owner.get(name) else {
                return Err(Error::NameDoesNotExists);
            };
            if caller != &owner {
                return Err(Error::NotOwner);
            }

            self.name_to_multilocation.insert(name, loc);
            Ok(())
        }

        fn ensure_admin(&self) -> Result<(), Error> {
            if self.env().caller() != self.admin {
                Err(Error::NotAdmin)?;
            }
            Ok(())
        }

        fn ensure_handler(&self) -> Result<(), Error> {
            if self.env().caller() != self.handler {
                Err(Error::NotHandler)?;
            }
            Ok(())
        }
    }
}
