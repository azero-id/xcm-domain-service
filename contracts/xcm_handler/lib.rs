#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod xcm_handler {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use scale::Encode;
    use utils::make_xcm_contract_call;
    use xcm::v3::prelude::*;
    use xcm::VersionedMultiLocation;

    pub type MultilocationEncoded = (u8, u32, AccountId); // (Parent, Parachain, AccountId)
    pub type ReadInterfaceEncoded = Vec<u8>;
    pub type TicketId = u128;

    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ReadInterface {
        Owner(Option<AccountId>),
        Address(Option<VersionedMultiLocation>),
    }

    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotAdmin,
        UnknownCaller,
        CallRuntimeFailed,
        InkEnvError,
        DomainService(u8),
    }

    impl From<ink::env::Error> for Error {
        fn from(e: ink::env::Error) -> Self {
            match e {
                ink::env::Error::CallRuntimeFailed => Error::CallRuntimeFailed,
                _ => Error::InkEnvError,
            }
        }
    }

    #[ink::trait_definition]
    pub trait DomainService {
        #[ink(message, selector = 0x07fcd0b1)]
        fn get_owner(&self, name: String) -> Option<AccountId>;

        #[ink(message, selector = 0xd259f7ba)]
        fn get_address(&self, name: String) -> Option<VersionedMultiLocation>;

        #[ink(message, selector = 0x56c905c6)]
        fn xcm_register_name(&mut self, caller: AccountId, name: String) -> Result<(), u8>;

        #[ink(message, selector = 0xf874dc03)]
        fn xcm_transfer_name(
            &mut self,
            caller: AccountId,
            name: String,
            receiver: AccountId,
        ) -> Result<(), u8>;

        #[ink(message, selector = 0xa06e9770)]
        fn xcm_set_address(
            &mut self,
            caller: AccountId,
            name: String,
            loc: MultilocationEncoded,
        ) -> Result<(), u8>;
    }

    #[ink(storage)]
    pub struct XcmHandler {
        admin: AccountId,
        domain_service: ink::contract_ref!(DomainService),
        xc_contracts: Mapping<AccountId, MultilocationEncoded>,
    }

    impl XcmHandler {
        #[ink(constructor)]
        pub fn new(admin: AccountId, domain_service_addr: AccountId) -> Self {
            Self {
                admin,
                domain_service: domain_service_addr.into(),
                xc_contracts: Mapping::default(),
            }
        }

        #[ink(message)]
        pub fn add_xc_contract(
            &mut self,
            xc_contract_soac: AccountId,
            origin_path: MultilocationEncoded,
        ) -> Result<(), Error> {
            self.ensure_admin()?;

            // check to ensure only supported Location types are added
            Self::resolve_location(&origin_path)?;
            self.xc_contracts.insert(xc_contract_soac, &origin_path);

            Ok(())
        }

        #[ink(message)]
        pub fn get_owner(
            &mut self,
            tid: TicketId,
            name: String,
        ) -> Result<Option<AccountId>, Error> {
            let origin_path = self.auth_caller()?;

            let output = self.domain_service.get_owner(name);
            let read_interface = ReadInterface::Owner(output);
            Self::send_response_back(&origin_path, &tid, &read_interface)?;

            Ok(output)
        }

        #[ink(message)]
        pub fn get_address(
            &mut self,
            tid: TicketId,
            name: String,
        ) -> Result<Option<VersionedMultiLocation>, Error> {
            let origin_path = self.auth_caller()?;

            let output = self.domain_service.get_address(name);

            // @todo: Verify its correctness
            let re_anchored_loc = match &output {
                Some(rs) => Some(self.reanchor_loc(rs, &origin_path)?),
                None => None,
            };

            let read_interface = ReadInterface::Address(re_anchored_loc);
            Self::send_response_back(&origin_path, &tid, &read_interface)?;

            Ok(output)
        }

        #[ink(message)]
        pub fn register_name(&mut self, caller: AccountId, name: String) -> Result<(), Error> {
            let origin_path = self.auth_caller()?;

            let caller_soac = self.interchain_account(&origin_path, &caller);
            self.domain_service
                .xcm_register_name(caller_soac, name)
                .map_err(Error::DomainService)
        }

        #[ink(message)]
        pub fn transfer_name(
            &mut self,
            caller: AccountId,
            name: String,
            receiver: AccountId,
        ) -> Result<(), Error> {
            let origin_path = self.auth_caller()?;

            let caller_soac = self.interchain_account(&origin_path, &caller);
            self.domain_service
                .xcm_transfer_name(caller_soac, name, receiver)
                .map_err(Error::DomainService)
        }

        #[ink(message)]
        pub fn set_address(
            &mut self,
            caller: AccountId,
            name: String,
            loc: MultilocationEncoded,
        ) -> Result<(), Error> {
            let origin_path = self.auth_caller()?;

            let caller_soac = self.interchain_account(&origin_path, &caller);

            // @todo: Re-anchor loc
            self.domain_service
                .xcm_set_address(caller_soac, name, loc)
                .map_err(Error::DomainService)
        }

        fn send_response_back(
            location: &MultilocationEncoded,
            tid: &TicketId,
            read_interface: &ReadInterface,
        ) -> Result<(), Error> {
            let (path_to_chain, contract_address) = Self::resolve_location(&location)?;

            let selector = ink::selector_bytes!("accept_response");
            let encoded_response: ReadInterfaceEncoded = read_interface.encode();

            make_xcm_contract_call::<Self>(
                path_to_chain.into(),
                contract_address,
                (selector, tid, encoded_response).encode(),
                0,
                None,
            )
            .map_err(Into::into)
        }

        fn resolve_location(
            location: &MultilocationEncoded,
        ) -> Result<(MultiLocation, AccountId), Error> {
            let &(parents, para_id, contract_addr) = location;
            let path_to_chain: MultiLocation = MultiLocation::new(parents, Parachain(para_id));

            Ok((path_to_chain, contract_addr))
        }

        // @todo: Implementation pending
        fn reanchor_loc(
            &self,
            loc: &VersionedMultiLocation,
            relative_to: &MultilocationEncoded,
        ) -> Result<VersionedMultiLocation, Error> {
            Ok(loc.clone())
        }

        // Gives us the control to have our own form of interchain account
        // Ideally should comply w/ the chains' Sovereign Account for consistency
        fn interchain_account(
            &self,
            _origin_path: &MultilocationEncoded,
            origin: &AccountId,
        ) -> AccountId {
            *origin // Alias Mode ON
        }

        fn auth_caller(&self) -> Result<MultilocationEncoded, Error> {
            let caller = self.env().caller();

            match self.xc_contracts.get(caller) {
                Some(loc) => Ok(loc),
                None => Err(Error::UnknownCaller),
            }
        }

        fn ensure_admin(&self) -> Result<(), Error> {
            if self.env().caller() != self.admin {
                Err(Error::NotAdmin)?;
            }
            Ok(())
        }
    }
}
