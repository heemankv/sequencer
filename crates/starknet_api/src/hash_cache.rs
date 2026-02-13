use dashmap::DashMap;
use starknet_types_core::felt::Felt;
use std::sync::LazyLock;

const SN_KECCAK_CACHE_CAPACITY: usize = 8 * 1024;
const SN_KECCAK_ORIGIN_CACHE_CAPACITY: usize = 8 * 1024;
const PEDERSEN_PAIR_CACHE_CAPACITY: usize = 8 * 1024;
const PEDERSEN_ARRAY_CACHE_CAPACITY: usize = 2 * 1024;
const POSEIDON_ARRAY_CACHE_CAPACITY: usize = 2 * 1024;

static SN_KECCAK_CACHE: LazyLock<DashMap<Vec<u8>, Felt>> = LazyLock::new(DashMap::new);
static SN_KECCAK_ORIGIN_CACHE: LazyLock<DashMap<Felt, String>> = LazyLock::new(DashMap::new);
static PEDERSEN_PAIR_CACHE: LazyLock<DashMap<(Felt, Felt), Felt>> = LazyLock::new(DashMap::new);
static PEDERSEN_ARRAY_CACHE: LazyLock<DashMap<Vec<Felt>, Felt>> = LazyLock::new(DashMap::new);
static POSEIDON_ARRAY_CACHE: LazyLock<DashMap<Vec<Felt>, Felt>> = LazyLock::new(DashMap::new);

#[inline]
pub fn sn_keccak_get(data: &[u8]) -> Option<Felt> {
    SN_KECCAK_CACHE.get(data).map(|v| *v)
}

#[inline]
pub fn sn_keccak_insert(data: &[u8], value: Felt) {
    if SN_KECCAK_CACHE.len() >= SN_KECCAK_CACHE_CAPACITY {
        SN_KECCAK_CACHE.clear();
    }
    SN_KECCAK_CACHE.insert(data.to_vec(), value);
}

#[inline]
pub fn sn_keccak_origin_get(value: Felt) -> Option<String> {
    SN_KECCAK_ORIGIN_CACHE.get(&value).map(|v| v.value().clone())
}

#[inline]
pub fn sn_keccak_origin_insert(value: Felt, data_hex: &str) {
    if SN_KECCAK_ORIGIN_CACHE.len() >= SN_KECCAK_ORIGIN_CACHE_CAPACITY {
        SN_KECCAK_ORIGIN_CACHE.clear();
    }
    SN_KECCAK_ORIGIN_CACHE.insert(value, data_hex.to_string());
}

#[inline]
pub fn pedersen_pair_get(left: Felt, right: Felt) -> Option<Felt> {
    PEDERSEN_PAIR_CACHE.get(&(left, right)).map(|v| *v)
}

#[inline]
pub fn pedersen_pair_insert(left: Felt, right: Felt, value: Felt) {
    if PEDERSEN_PAIR_CACHE.len() >= PEDERSEN_PAIR_CACHE_CAPACITY {
        PEDERSEN_PAIR_CACHE.clear();
    }
    PEDERSEN_PAIR_CACHE.insert((left, right), value);
}

#[inline]
pub fn pedersen_array_get(values: &[Felt]) -> Option<Felt> {
    PEDERSEN_ARRAY_CACHE.get(values).map(|v| *v)
}

#[inline]
pub fn pedersen_array_insert(values: &[Felt], value: Felt) {
    if PEDERSEN_ARRAY_CACHE.len() >= PEDERSEN_ARRAY_CACHE_CAPACITY {
        PEDERSEN_ARRAY_CACHE.clear();
    }
    PEDERSEN_ARRAY_CACHE.insert(values.to_vec(), value);
}

#[inline]
pub fn poseidon_array_get(values: &[Felt]) -> Option<Felt> {
    POSEIDON_ARRAY_CACHE.get(values).map(|v| *v)
}

#[inline]
pub fn poseidon_array_insert(values: &[Felt], value: Felt) {
    if POSEIDON_ARRAY_CACHE.len() >= POSEIDON_ARRAY_CACHE_CAPACITY {
        POSEIDON_ARRAY_CACHE.clear();
    }
    POSEIDON_ARRAY_CACHE.insert(values.to_vec(), value);
}
