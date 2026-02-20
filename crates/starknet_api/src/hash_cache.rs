use dashmap::DashMap;
use starknet_types_core::felt::Felt;
use std::sync::LazyLock;

use crate::hash_agg;

const SN_KECCAK_CACHE_CAPACITY: usize = 8 * 1024;
const PEDERSEN_PAIR_CACHE_CAPACITY: usize = 8 * 1024;
const PEDERSEN_ARRAY_CACHE_CAPACITY: usize = 2 * 1024;
const POSEIDON_ARRAY_CACHE_CAPACITY: usize = 2 * 1024;

static HASH_CACHE_ENABLED: LazyLock<bool> = LazyLock::new(|| {
    let value = std::env::var("BLOCKIFIER_HASH_CACHE_ENABLED").unwrap_or_default();
    if value.is_empty() {
        return false;
    }
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => true,
        _ => false,
    }
});

static SN_KECCAK_CACHE: LazyLock<DashMap<Vec<u8>, Felt>> = LazyLock::new(DashMap::new);
static PEDERSEN_PAIR_CACHE: LazyLock<DashMap<(Felt, Felt), Felt>> = LazyLock::new(DashMap::new);
static PEDERSEN_ARRAY_CACHE: LazyLock<DashMap<Vec<Felt>, Felt>> = LazyLock::new(DashMap::new);
static POSEIDON_ARRAY_CACHE: LazyLock<DashMap<Vec<Felt>, Felt>> = LazyLock::new(DashMap::new);

#[inline]
pub fn sn_keccak_get(data: &[u8]) -> Option<Felt> {
    if !*HASH_CACHE_ENABLED {
        hash_agg::record_sn_keccak_call(data, false);
        return None;
    }
    let cached = SN_KECCAK_CACHE.get(data).map(|v| *v);
    hash_agg::record_sn_keccak_call(data, cached.is_some());
    cached
}

#[inline]
pub fn sn_keccak_insert(data: &[u8], value: Felt) {
    if !*HASH_CACHE_ENABLED {
        return;
    }
    if SN_KECCAK_CACHE.len() >= SN_KECCAK_CACHE_CAPACITY {
        SN_KECCAK_CACHE.clear();
    }
    SN_KECCAK_CACHE.insert(data.to_vec(), value);
}

#[inline]
pub fn pedersen_pair_get(left: Felt, right: Felt) -> Option<Felt> {
    if !*HASH_CACHE_ENABLED {
        hash_agg::record_pedersen_pair_call(left, right, false);
        return None;
    }
    let cached = PEDERSEN_PAIR_CACHE.get(&(left, right)).map(|v| *v);
    hash_agg::record_pedersen_pair_call(left, right, cached.is_some());
    cached
}

#[inline]
pub fn pedersen_pair_insert(left: Felt, right: Felt, value: Felt) {
    if !*HASH_CACHE_ENABLED {
        return;
    }
    if PEDERSEN_PAIR_CACHE.len() >= PEDERSEN_PAIR_CACHE_CAPACITY {
        PEDERSEN_PAIR_CACHE.clear();
    }
    PEDERSEN_PAIR_CACHE.insert((left, right), value);
}

#[inline]
pub fn pedersen_array_get(values: &[Felt]) -> Option<Felt> {
    if !*HASH_CACHE_ENABLED {
        hash_agg::record_pedersen_array_call(values, false);
        return None;
    }
    let cached = PEDERSEN_ARRAY_CACHE.get(values).map(|v| *v);
    hash_agg::record_pedersen_array_call(values, cached.is_some());
    cached
}

#[inline]
pub fn pedersen_array_insert(values: &[Felt], value: Felt) {
    if !*HASH_CACHE_ENABLED {
        return;
    }
    if PEDERSEN_ARRAY_CACHE.len() >= PEDERSEN_ARRAY_CACHE_CAPACITY {
        PEDERSEN_ARRAY_CACHE.clear();
    }
    PEDERSEN_ARRAY_CACHE.insert(values.to_vec(), value);
}

#[inline]
pub fn poseidon_array_get(values: &[Felt]) -> Option<Felt> {
    if !*HASH_CACHE_ENABLED {
        hash_agg::record_poseidon_call(values);
        return None;
    }
    let cached = POSEIDON_ARRAY_CACHE.get(values).map(|v| *v);
    hash_agg::record_poseidon_call(values);
    cached
}

#[inline]
pub fn poseidon_array_insert(values: &[Felt], value: Felt) {
    if !*HASH_CACHE_ENABLED {
        return;
    }
    if POSEIDON_ARRAY_CACHE.len() >= POSEIDON_ARRAY_CACHE_CAPACITY {
        POSEIDON_ARRAY_CACHE.clear();
    }
    POSEIDON_ARRAY_CACHE.insert(values.to_vec(), value);
}
