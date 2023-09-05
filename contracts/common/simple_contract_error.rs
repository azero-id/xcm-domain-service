#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum SimpleContractError {
    NotAdmin,
    NotHandler,
}
