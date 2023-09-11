use crate::Weight;

const WEIGHT_PER_INSTRUCTION: Weight = Weight::from_parts(1_000, 1_000);
const WEIGHT_REF_TIME_PER_SECOND: u64 = 1_000_000_000_000;
const WEIGHT_PROOF_SIZE_PER_MB: u64 = 1024 * 1024;
const UNITS_PER_SECOND: u128 = 1_000_000_000_000;
const UNITS_PER_MB: u128 = 1024 * 1024;

pub fn estimate_message_fee(number_of_instructions: u64) -> u128 {
    let weight = estimate_weight(number_of_instructions);
    estimate_fee_for_weight(weight)
}

pub fn estimate_weight(number_of_instructions: u64) -> Weight {
    WEIGHT_PER_INSTRUCTION.saturating_mul(number_of_instructions)
}

pub fn estimate_fee_for_weight(weight: Weight) -> u128 {
    UNITS_PER_SECOND * (weight.ref_time() as u128) / (WEIGHT_REF_TIME_PER_SECOND as u128)
        + UNITS_PER_MB * (weight.proof_size() as u128) / (WEIGHT_PROOF_SIZE_PER_MB as u128)
}