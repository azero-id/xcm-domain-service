#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod xc_who_am_i {
    use common::{
        call_encodings::{ContractsCall, RuntimeCall, XcmCall},
        xcm_utils::*,
        Weight,
    };
    use ink::env::Error as EnvError;
    use ink::prelude::{boxed::Box, vec};
    use xcm::{v3::prelude::*, VersionedXcm};

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum RuntimeError {
        CallRuntimeFailed,
    }

    impl From<EnvError> for RuntimeError {
        fn from(e: EnvError) -> Self {
            match e {
                EnvError::CallRuntimeFailed => RuntimeError::CallRuntimeFailed,
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

            let gas = Weight::from_parts(10_000_000_000, 10_000_000_000);
            let est_wt = estimate_weight(4) + gas * 2;
            let fee = estimate_fee_for_weight(est_wt);

            let call = RuntimeCall::Contracts(ContractsCall::Call {
                dest: self.contract,
                value: 0,
                gas_limit: gas,
                storage_deposit_limit: None,
                data: sel_walk_in.to_vec(),
            });

            let message: Xcm<()> = Xcm(vec![
                WithdrawAsset(vec![(Parent, fee).into()].into()),
                BuyExecution {
                    fees: (Parent, fee).into(),
                    weight_limit: WeightLimit::Unlimited,
                },
                Transact {
                    origin_kind: OriginKind::SovereignAccount,
                    require_weight_at_most: gas * 2,
                    call: scale::Encode::encode(&call).into(),
                },
            ]);

            self.env()
                .call_runtime(&RuntimeCall::Xcm(XcmCall::Send {
                    dest: Box::new((Parent, Parachain(1)).into()),
                    message: Box::new(VersionedXcm::V3(message)),
                }))
                .map_err(Into::into)
        }
    }
}
