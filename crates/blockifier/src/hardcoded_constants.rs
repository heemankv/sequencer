//! Hardcoded values for fast-path execution experiments.
//!
//! These constants mirror `madara/crates/client/rust-exec/src/constants.rs` and are intentionally
//! centralized so we can return consistent fee/resources without computing them.

use starknet_api::execution_resources::{GasAmount, GasVector};
use starknet_api::transaction::fields::Fee;

// NOTE: These numbers are currently hardcoded based on an empirical Blockifier run of
// `settle_trade_v3`. They are a temporary bridge until we can produce receipts/resources
// natively with full fidelity.
pub const SETTLE_TRADE_V3_FIXED_FEE_AMOUNT: u128 = 0x2639_288b_e94;
pub const SETTLE_TRADE_V3_FIXED_L1_GAS: u64 = 0;
pub const SETTLE_TRADE_V3_FIXED_L1_DATA_GAS: u64 = 576;
pub const SETTLE_TRADE_V3_FIXED_L2_GAS: u64 = 25_696_375;

// Bouncer weights used for block closure. These match rust-exec hardcoded deltas.
pub const SETTLE_TRADE_V3_BOUNCER_FIRST_SIERRA_GAS: u64 = 600_124_824;
pub const SETTLE_TRADE_V3_BOUNCER_FIRST_PROVING_GAS: u64 = 606_943_554;
pub const SETTLE_TRADE_V3_BOUNCER_STEADY_SIERRA_GAS: u64 = 37_282_620;
pub const SETTLE_TRADE_V3_BOUNCER_STEADY_PROVING_GAS: u64 = 41_409_414;

pub fn hardcoded_fee() -> Fee {
    Fee(SETTLE_TRADE_V3_FIXED_FEE_AMOUNT)
}

pub fn hardcoded_gas_vector() -> GasVector {
    GasVector {
        l1_gas: GasAmount(SETTLE_TRADE_V3_FIXED_L1_GAS),
        l1_data_gas: GasAmount(SETTLE_TRADE_V3_FIXED_L1_DATA_GAS),
        l2_gas: GasAmount(SETTLE_TRADE_V3_FIXED_L2_GAS),
    }
}

pub fn hardcoded_da_gas_vector() -> GasVector {
    GasVector {
        l1_gas: GasAmount(0),
        l1_data_gas: GasAmount(SETTLE_TRADE_V3_FIXED_L1_DATA_GAS),
        l2_gas: GasAmount(0),
    }
}

pub fn hardcoded_bouncer_gas(is_first: bool) -> (GasAmount, GasAmount) {
    if is_first {
        (
            GasAmount(SETTLE_TRADE_V3_BOUNCER_FIRST_SIERRA_GAS),
            GasAmount(SETTLE_TRADE_V3_BOUNCER_FIRST_PROVING_GAS),
        )
    } else {
        (
            GasAmount(SETTLE_TRADE_V3_BOUNCER_STEADY_SIERRA_GAS),
            GasAmount(SETTLE_TRADE_V3_BOUNCER_STEADY_PROVING_GAS),
        )
    }
}
