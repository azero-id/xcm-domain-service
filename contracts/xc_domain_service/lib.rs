#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod xc_domain_service {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use scale::{Decode, Encode};
    use utils::make_xcm_contract_call;
    use utils::MultilocationEncoded;
    use xcm::v3::prelude::*;

    const PATH_TO_HOST_CHAIN: MultiLocation = MultiLocation {
        parents: 1,
        interior: X1(Parachain(1)),
    };

    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ReadInterface {
        Owner(Option<AccountId>),
        Address(Option<xcm::VersionedMultiLocation>),
    }

    pub type ReadInterfaceEncoded = Vec<u8>;
    pub type TicketId = u128;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotHandler,
        PaymentNotReceived,
        CallRuntimeFailed,
        InvalidTicketId,
        TicketIdMismatch,
        AwaitingResponse,
        DuplicateResponse,
        FailedToDecodeResponse,
        InkEnvError,
    }

    impl From<ink::env::Error> for Error {
        fn from(e: ink::env::Error) -> Self {
            match e {
                ink::env::Error::CallRuntimeFailed => Error::CallRuntimeFailed,
                _ => Error::InkEnvError,
            }
        }
    }

    #[ink(event)]
    pub struct ResponseReceived {
        #[ink(topic)]
        ticket_id: TicketId,
    }
    #[ink(storage)]
    pub struct XcDomainService {
        xcm_handler: AccountId,
        xcm_handler_soac: AccountId, // Try computing it on-chain
        ticket_count: TicketId,
        ticket_to_response: Mapping<TicketId, ReadInterfaceEncoded>,
        custom_weight: Option<(u64, u64)>,
    }

    impl XcDomainService {
        #[ink(constructor)]
        pub fn new(
            xcm_handler: AccountId, 
            xcm_handler_soac: AccountId,
            custom_weight: Option<(u64, u64)>,
        ) -> Self {
            Self {
                xcm_handler,
                xcm_handler_soac,
                ticket_count: 0,
                ticket_to_response: Mapping::default(),
                custom_weight,
            }
        }

        /// Returns (xcm-handler, xcm-handler-soac) addresses
        #[ink(message)]
        pub fn get_handler_details(&self) -> (AccountId, AccountId) {
            (self.xcm_handler, self.xcm_handler_soac)
        }

        #[ink(message)]
        pub fn get_ticket_count(&self) -> TicketId {
            self.ticket_count
        }

        /** Async getters STARTS here */

        /// Requests for the owner details of the given name.
        /// Returns `TicketId` for the request. 
        /// Use this `TicketId` with `retrieve_owner` to get the details.
        #[ink(message)]
        pub fn get_owner(&mut self, name: String) -> Result<TicketId, Error> {
            let tid = self.ticket_count;

            let selector = ink::selector_bytes!("get_owner");
            let payload = (selector, tid, name).encode();
            self.call_handler(payload)?;

            self.ticket_count += 1;
            Ok(tid)
        }

        /// Requests for the resolving address details of the given name.
        /// Returns `TicketId` for the request. 
        /// Use this `TicketId` with `retrieve_address` to get the details.
        #[ink(message)]
        pub fn get_address(&mut self, name: String) -> Result<TicketId, Error> {
            let tid = self.ticket_count;

            let selector = ink::selector_bytes!("get_address");
            let payload = (selector, tid, name).encode();
            self.call_handler(payload)?;

            self.ticket_count += 1;
            Ok(tid)
        }

        /** Async getters ENDS here */

        /** Getters request fulfill STARTS here */

        /// Get the owner details associated with the TicketId (if valid)
        #[ink(message)]
        pub fn retrieve_owner(&self, tid: TicketId) -> Result<Option<AccountId>, Error> {
            match self.read_response(tid)? {
                ReadInterface::Owner(rs) => Ok(rs),
                _ => Err(Error::TicketIdMismatch),
            }
        }

        /// Get the address details associated with the TicketId (if valid)
        #[ink(message)]
        pub fn retrieve_address(
            &self,
            tid: TicketId,
        ) -> Result<Option<xcm::VersionedMultiLocation>, Error> {
            match self.read_response(tid)? {
                ReadInterface::Address(rs) => Ok(rs),
                _ => Err(Error::TicketIdMismatch),
            }
        }

        /// Returns whole response associated with the tid (if valid)
        #[ink(message)]
        pub fn read_response(&self, tid: TicketId) -> Result<ReadInterface, Error> {
            if tid >= self.ticket_count {
                Err(Error::InvalidTicketId)?
            }

            let Some(response) = self.ticket_to_response.get(tid) else {
                Err(Error::AwaitingResponse)?
            };
            ReadInterface::decode(&mut &response[..]).map_err(|_| Error::FailedToDecodeResponse)
        }

        /// Returns the response associated with the tid in encoded format
        #[ink(message)]
        pub fn read_raw_response(&self, tid: TicketId) -> Option<ReadInterfaceEncoded> {
            self.ticket_to_response.get(tid)
        }

        /** Getters request fulfill ENDS here */

        /** Async setters STARTS here */

        /// @note For simplicity, Assumption is made that the name will be successfully registered
        /// and therefore refund case is not handled here!
        #[ink(message, payable)]
        pub fn register_name(&mut self, name: String) -> Result<(), Error> {
            if self.env().transferred_value() < 80 {
                return Err(Error::PaymentNotReceived);
            }

            let selector = ink::selector_bytes!("register_name");
            let caller = self.env().caller();
            let payload = (selector, caller, name).encode();

            self.call_handler(payload)
        }

        #[ink(message)]
        pub fn transfer_name(&mut self, name: String, receiver: AccountId) -> Result<(), Error> {
            let selector = ink::selector_bytes!("transfer_name");
            let caller = self.env().caller();
            let payload = (selector, caller, name, receiver).encode();

            self.call_handler(payload)
        }

        /// Allow setting the resolving address in `MultiLocation` format
        #[ink(message)]
        pub fn set_address(
            &mut self,
            name: String,
            loc: MultilocationEncoded,
        ) -> Result<(), Error> {
            let selector = ink::selector_bytes!("set_address");
            let caller = self.env().caller();
            let payload = (selector, caller, name, loc).encode();

            self.call_handler(payload)
        }

        /** Async setters ENDS here */

        /// For xcm-handler response only!
        #[ink(message)]
        pub fn accept_response(
            &mut self,
            tid: TicketId,
            response: ReadInterfaceEncoded,
        ) -> Result<(), Error> {
            self.ensure_handler()?;

            if tid >= self.ticket_count {
                Err(Error::InvalidTicketId)?
            } else if self.ticket_to_response.contains(tid) {
                Err(Error::DuplicateResponse)?
            }

            self.ticket_to_response.insert(tid, &response);

            // Emit event to announce response availability
            self.env().emit_event(ResponseReceived { ticket_id: tid });

            Ok(())
        }

        fn call_handler(&mut self, payload: Vec<u8>) -> Result<(), Error> {
            let wt = self.custom_weight.map(|(x,y)| utils::Weight::from_parts(x,y));
            
            make_xcm_contract_call::<Self>(
                PATH_TO_HOST_CHAIN.into(),
                self.xcm_handler,
                payload,
                0,
                wt,
            )
            .map_err(Into::into)
        }

        fn ensure_handler(&self) -> Result<(), Error> {
            if self.env().caller() != self.xcm_handler_soac {
                Err(Error::NotHandler)?
            }
            Ok(())
        }
    }
}
