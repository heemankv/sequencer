use sha3::{Digest, Keccak256};
use starknet_types_core::felt::{Felt, NonZeroFelt};
use starknet_types_core::hash::{Pedersen, StarkHash};
use std::time::Instant;

use crate::abi::constants;
use crate::core::{ContractAddress, EntryPointSelector, PatriciaKey, L2_ADDRESS_UPPER_BOUND};
use crate::hash_cache;
use crate::hash_metrics;
use crate::state::StorageKey;

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

#[cfg(test)]
#[path = "abi_utils_test.rs"]
mod test;

/// A variant of eth-keccak that computes a value that fits in a Starknet field element.
pub fn starknet_keccak(data: &[u8]) -> Felt {
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
    if let Some(cached) = hash_cache::sn_keccak_get(data) {
        if let Some(start) = log_start {
            let total_us = start.elapsed().as_micros();
            let data_hex = bytes_to_hex(data);
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
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let mut result: [u8; 32] = hasher.finalize().into();

    // Truncate result to 250 bits.
    *result.first_mut().unwrap() &= 3;
    let felt = Felt::from_bytes_be(&result);
    hash_cache::sn_keccak_insert(data, felt);
    if let Some(start) = log_start {
        let total_us = start.elapsed().as_micros();
        let data_hex = bytes_to_hex(data);
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

/// Returns an entry point selector, given its name.
pub fn selector_from_name(entry_point_name: &str) -> EntryPointSelector {
    static DEFAULT_ENTRY_POINTS: [&str; 2] =
        [constants::DEFAULT_ENTRY_POINT_NAME, constants::DEFAULT_L1_ENTRY_POINT_NAME];

    // The default entry points selector is not being mapped in the usual way in order to save
    // computations in the OS, and to avoid encoding the default entry point names there.
    if DEFAULT_ENTRY_POINTS.contains(&entry_point_name) {
        EntryPointSelector(Felt::from(constants::DEFAULT_ENTRY_POINT_SELECTOR))
    } else {
        EntryPointSelector(starknet_keccak(entry_point_name.as_bytes()))
    }
}

/// Returns the storage address of a Starknet storage variable given its name and arguments.
pub fn get_storage_var_address(storage_var_name: &str, args: &[Felt]) -> StorageKey {
    let storage_var_name_hash = starknet_keccak(storage_var_name.as_bytes());
    if hash_logs_enabled() {
        let args_hex = args
            .iter()
            .map(|v| format!("{:#x}", v))
            .collect::<Vec<_>>()
            .join(", ");
        tracing::info!(
            "blockifier-starknet-api-exec: storage_var_base: name={} base={:#x} args=[{}]",
            storage_var_name,
            storage_var_name_hash,
            args_hex
        );
    }

    let mut storage_key_hash = storage_var_name_hash;
    for arg in args {
        let log_start = if hash_logs_enabled() {
            Some(Instant::now())
        } else {
            None
        };
        let timing_start = if hash_metrics::hash_timing_enabled() {
            Some(Instant::now())
        } else {
            None
        };
        let left = storage_key_hash;
        let right = *arg;
        if let Some(cached) = hash_cache::pedersen_pair_get(left, right) {
            storage_key_hash = cached;
            if let Some(start) = log_start {
                let total_us = start.elapsed().as_micros();
                if let Some(values) = hash_cache::sn_keccak_origin_get(left) {
                    tracing::info!(
                        "blockifier-starknet-api-exec: pedersen(left_origin_sn_keccak): left={:#x} data={}",
                        left,
                        values
                    );
                }
                tracing::info!(
                    "blockifier-starknet-api-exec: pedersen(hash): cache-hit : left={:#x} right={:#x} : result={:#x} : total_us={}",
                    left,
                    right,
                    storage_key_hash,
                    total_us
                );
            }
            if let Some(start) = timing_start {
                hash_metrics::record_pedersen(start.elapsed().as_micros() as u64);
            }
            continue;
        }
        storage_key_hash = Pedersen::hash(&storage_key_hash, arg);
        hash_cache::pedersen_pair_insert(left, right, storage_key_hash);
        if let Some(start) = log_start {
            let total_us = start.elapsed().as_micros();
            if let Some(values) = hash_cache::sn_keccak_origin_get(left) {
                tracing::info!(
                    "blockifier-starknet-api-exec: pedersen(left_origin_sn_keccak): left={:#x} data={}",
                    left,
                    values
                );
            }
            tracing::info!(
                "blockifier-starknet-api-exec: pedersen(hash): cache-miss : left={:#x} right={:#x} : result={:#x} : total_us={}",
                left,
                right,
                storage_key_hash,
                total_us
            );
        }
        if let Some(start) = timing_start {
            hash_metrics::record_pedersen(start.elapsed().as_micros() as u64);
        }
    }

    let storage_key = storage_key_hash
        .mod_floor(&NonZeroFelt::from_raw(Felt::from(*L2_ADDRESS_UPPER_BOUND).to_raw()));

    StorageKey(
        PatriciaKey::try_from(storage_key)
            .expect("Should be within bounds as retrieved mod L2_ADDRESS_UPPER_BOUND."),
    )
}

/// Returns the storage key inside the fee token corresponding to the first storage cell where the
/// balance of contract_address is stored. Note that the reference implementation of an ERC20 stores
/// the balance in two consecutive storage cells.
pub fn get_fee_token_var_address(contract_address: ContractAddress) -> StorageKey {
    get_storage_var_address("ERC20_balances", &[*contract_address.0.key()])
}
