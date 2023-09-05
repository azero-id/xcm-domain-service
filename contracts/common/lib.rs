#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod simple_contract_error;

pub use simple_contract_error::SimpleContractError;
pub type MultiAddress = ink::primitives::AccountId;
