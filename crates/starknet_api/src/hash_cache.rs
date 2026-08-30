use std::sync::{
    LazyLock,
    atomic::{AtomicBool, Ordering},
};

use dashmap::DashMap;
use starknet_types_core::felt::Felt;

const SN_KECCAK_CACHE_CAPACITY: usize = 8 * 1024;
const PEDERSEN_PAIR_CACHE_CAPACITY: usize = 8 * 1024;
const PEDERSEN_ARRAY_CACHE_CAPACITY: usize = 2 * 1024;
const POSEIDON_ARRAY_CACHE_CAPACITY: usize = 2 * 1024;

static HASH_CACHE_ENABLED: AtomicBool = AtomicBool::new(false);
static SN_KECCAK_CACHE: LazyLock<DashMap<Vec<u8>, Felt>> = LazyLock::new(DashMap::new);
static PEDERSEN_PAIR_CACHE: LazyLock<DashMap<(Felt, Felt), Felt>> = LazyLock::new(DashMap::new);
static PEDERSEN_ARRAY_CACHE: LazyLock<DashMap<Vec<Felt>, Felt>> = LazyLock::new(DashMap::new);
static POSEIDON_ARRAY_CACHE: LazyLock<DashMap<Vec<Felt>, Felt>> = LazyLock::new(DashMap::new);

/// Enables or disables process-wide Starknet hash memoization.
///
/// Configure this once during process startup, before execution workers begin
/// handling transactions.
pub fn set_hash_cache_enabled(enabled: bool) {
    HASH_CACHE_ENABLED.store(enabled, Ordering::Relaxed);
    if !enabled {
        SN_KECCAK_CACHE.clear();
        PEDERSEN_PAIR_CACHE.clear();
        PEDERSEN_ARRAY_CACHE.clear();
        POSEIDON_ARRAY_CACHE.clear();
    }
}

fn enabled() -> bool {
    HASH_CACHE_ENABLED.load(Ordering::Relaxed)
}

fn insert<K, V>(cache: &DashMap<K, V>, capacity: usize, key: K, value: V)
where
    K: Eq + std::hash::Hash,
{
    if !enabled() {
        return;
    }
    if cache.len() >= capacity {
        cache.clear();
    }
    cache.insert(key, value);
}

pub(crate) fn sn_keccak_get(data: &[u8]) -> Option<Felt> {
    if !enabled() {
        return None;
    }
    SN_KECCAK_CACHE.get(data).map(|value| *value)
}

pub(crate) fn sn_keccak_insert(data: &[u8], value: Felt) {
    insert(&SN_KECCAK_CACHE, SN_KECCAK_CACHE_CAPACITY, data.to_vec(), value);
}

pub(crate) fn pedersen_pair_get(left: Felt, right: Felt) -> Option<Felt> {
    if !enabled() {
        return None;
    }
    PEDERSEN_PAIR_CACHE.get(&(left, right)).map(|value| *value)
}

pub(crate) fn pedersen_pair_insert(left: Felt, right: Felt, value: Felt) {
    insert(&PEDERSEN_PAIR_CACHE, PEDERSEN_PAIR_CACHE_CAPACITY, (left, right), value);
}

pub(crate) fn pedersen_array_get(values: &[Felt]) -> Option<Felt> {
    if !enabled() {
        return None;
    }
    PEDERSEN_ARRAY_CACHE.get(values).map(|value| *value)
}

pub(crate) fn pedersen_array_insert(values: &[Felt], value: Felt) {
    insert(&PEDERSEN_ARRAY_CACHE, PEDERSEN_ARRAY_CACHE_CAPACITY, values.to_vec(), value);
}

pub(crate) fn poseidon_array_get(values: &[Felt]) -> Option<Felt> {
    if !enabled() {
        return None;
    }
    POSEIDON_ARRAY_CACHE.get(values).map(|value| *value)
}

pub(crate) fn poseidon_array_insert(values: &[Felt], value: Felt) {
    insert(&POSEIDON_ARRAY_CACHE, POSEIDON_ARRAY_CACHE_CAPACITY, values.to_vec(), value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_is_bypassed_when_disabled_and_reused_when_enabled() {
        let key = Felt::from(1_u8);
        let value = Felt::from(2_u8);

        set_hash_cache_enabled(false);
        pedersen_pair_insert(key, key, value);
        assert_eq!(pedersen_pair_get(key, key), None);

        set_hash_cache_enabled(true);
        pedersen_pair_insert(key, key, value);
        assert_eq!(pedersen_pair_get(key, key), Some(value));

        set_hash_cache_enabled(false);
    }
}
