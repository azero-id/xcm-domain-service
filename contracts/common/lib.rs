#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub mod call_encodings;
pub mod xcm_utils;
mod simple_contract_error;

pub use simple_contract_error::SimpleContractError;
pub use sp_weights::Weight;
pub type MultiAddress = ink::primitives::AccountId;
