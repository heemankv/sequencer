use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use starknet_types_core::felt::Felt;
use std::time::Instant;

use crate::hash_cache;
use crate::hash_metrics;

pub type StarkHash = Felt;

fn hash_logs_enabled() -> bool {
    std::env::var_os("BLOCKIFIER_HASH_LOGS").is_some()
}

fn bytes_to_hex(data: &[u8]) -> String {
    let mut out = String::with_capacity(2 + data.len() * 2);
    out.push_str("0x");
    for b in data {
        use std::fmt::Write;
        let _ = write!(out, "{:02x}", b);
    }
    out
}

#[derive(
    Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord,
)]
pub struct PoseidonHash(pub Felt);

/// Computes the first 250 bits of the Keccak256 hash, in order to fit into a field element.
pub fn starknet_keccak_hash(input: &[u8]) -> Felt {
    let timing_start = if hash_metrics::hash_timing_enabled() {
        Some(Instant::now())
    } else {
        None
    };
    let log_start = if hash_logs_enabled() {
        Some(Instant::now())
    } else {
        None
    };
    if let Some(cached) = hash_cache::sn_keccak_get(input) {
        if let Some(start) = log_start {
            let total_us = start.elapsed().as_micros();
            let data_hex = bytes_to_hex(input);
            tracing::info!(
                "blockifier-starknet-api-exec: sn_keccak(keccak): cache-hit : data={} : result={:#x} : total_us={}",
                data_hex,
                cached,
                total_us
            );
            hash_cache::sn_keccak_origin_insert(cached, &data_hex);
        }
        if let Some(start) = timing_start {
            hash_metrics::record_sn_keccak(start.elapsed().as_micros() as u64);
        }
        return cached;
    }
    let mut keccak = Keccak256::default();
    keccak.update(input);
    let mut hashed_bytes: [u8; 32] = keccak.finalize().into();
    hashed_bytes[0] &= 0b00000011_u8; // Discard the six MSBs.
    let felt = Felt::from_bytes_be(&hashed_bytes);
    hash_cache::sn_keccak_insert(input, felt);
    if let Some(start) = log_start {
        let total_us = start.elapsed().as_micros();
        let data_hex = bytes_to_hex(input);
        tracing::info!(
            "blockifier-starknet-api-exec: sn_keccak(keccak): cache-miss : data={} : result={:#x} : total_us={}",
            data_hex,
            felt,
            total_us
        );
        hash_cache::sn_keccak_origin_insert(felt, &data_hex);
    }
    if let Some(start) = timing_start {
        hash_metrics::record_sn_keccak(start.elapsed().as_micros() as u64);
    }
    felt
}

#[cfg(any(feature = "testing", test))]
pub struct FeltConverter;

#[cfg(any(feature = "testing", test))]
pub trait TryIntoFelt<V> {
    fn to_felt_unchecked(v: V) -> Felt;
}

macro_rules! impl_try_into_felt {
    ($type:ty) => {
        #[cfg(any(feature = "testing", test))]
        impl TryIntoFelt<$type> for FeltConverter {
            fn to_felt_unchecked(v: $type) -> Felt {
                Felt::from(v)
            }
        }
    };
}

impl_try_into_felt!(u128);
impl_try_into_felt!(u64);
impl_try_into_felt!(u32);
impl_try_into_felt!(u16);
impl_try_into_felt!(u8);

#[cfg(any(feature = "testing", test))]
impl TryIntoFelt<&str> for FeltConverter {
    fn to_felt_unchecked(v: &str) -> Felt {
        Felt::from_hex_unchecked(v)
    }
}

/// A utility macro to create a [`starknet_types_core::felt::Felt`] from an intergert or a hex
/// string representation.
#[cfg(any(feature = "testing", test))]
#[macro_export]
macro_rules! felt {
    ($s:expr) => {
        <$crate::hash::FeltConverter as $crate::hash::TryIntoFelt<_>>::to_felt_unchecked($s)
    };
}
