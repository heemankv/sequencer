use dashmap::DashMap;
use starknet_types_core::felt::Felt;
use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::LazyLock;

const SN_KECCAK_CACHE_CAPACITY: usize = 8 * 1024;
const SN_KECCAK_ORIGIN_CACHE_CAPACITY: usize = 8 * 1024;
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

static HASH_CALC_TOTALS_ENABLED: LazyLock<bool> = LazyLock::new(|| {
    let value = std::env::var("LOG_HASH_CALC_TOTALS").unwrap_or_default();
    if value.is_empty() {
        return false;
    }
    match value.to_ascii_lowercase().as_str() {
        "0" | "false" | "no" | "off" => false,
        _ => true,
    }
});

static SN_KECCAK_CACHE: LazyLock<DashMap<Vec<u8>, Felt>> = LazyLock::new(DashMap::new);
static SN_KECCAK_ORIGIN_CACHE: LazyLock<DashMap<Felt, String>> = LazyLock::new(DashMap::new);
static PEDERSEN_PAIR_CACHE: LazyLock<DashMap<(Felt, Felt), Felt>> = LazyLock::new(DashMap::new);
static PEDERSEN_ARRAY_CACHE: LazyLock<DashMap<Vec<Felt>, Felt>> = LazyLock::new(DashMap::new);
static POSEIDON_ARRAY_CACHE: LazyLock<DashMap<Vec<Felt>, Felt>> = LazyLock::new(DashMap::new);

#[derive(Debug, Clone, Copy, Default)]
pub struct HashCalcTotals {
    pub pedersen: u64,
    pub sn_keccak: u64,
    pub poseidon: u64,
}

#[derive(Default)]
struct HashCalcUniques {
    pedersen_pair: HashSet<(Felt, Felt)>,
    pedersen_array: HashSet<Vec<Felt>>,
    poseidon_array: HashSet<Vec<Felt>>,
    sn_keccak: HashSet<Vec<u8>>,
}

thread_local! {
    static HASH_CALC_TOTALS: RefCell<HashCalcTotals> = RefCell::new(HashCalcTotals::default());
    static HASH_CALC_UNIQUES: RefCell<HashCalcUniques> = RefCell::new(HashCalcUniques::default());
}

pub fn reset_hash_calc_stats() {
    if !*HASH_CALC_TOTALS_ENABLED {
        return;
    }
    HASH_CALC_TOTALS.with(|stats| {
        *stats.borrow_mut() = HashCalcTotals::default();
    });
    HASH_CALC_UNIQUES.with(|uniques| {
        let mut uniques = uniques.borrow_mut();
        uniques.pedersen_pair.clear();
        uniques.pedersen_array.clear();
        uniques.poseidon_array.clear();
        uniques.sn_keccak.clear();
    });
}

pub fn hash_calc_totals_snapshot() -> HashCalcTotals {
    HASH_CALC_TOTALS.with(|stats| *stats.borrow())
}

pub fn hash_calc_unique_counts() -> HashCalcTotals {
    HASH_CALC_UNIQUES.with(|uniques| {
        let uniques = uniques.borrow();
        HashCalcTotals {
            pedersen: (uniques.pedersen_pair.len() + uniques.pedersen_array.len()) as u64,
            sn_keccak: uniques.sn_keccak.len() as u64,
            poseidon: uniques.poseidon_array.len() as u64,
        }
    })
}

#[inline]
pub fn sn_keccak_get(data: &[u8]) -> Option<Felt> {
    if *HASH_CALC_TOTALS_ENABLED {
        HASH_CALC_TOTALS.with(|stats| {
            let mut stats = stats.borrow_mut();
            stats.sn_keccak = stats.sn_keccak.saturating_add(1);
        });
        HASH_CALC_UNIQUES.with(|uniques| {
            uniques.borrow_mut().sn_keccak.insert(data.to_vec());
        });
    }
    if !*HASH_CACHE_ENABLED {
        return None;
    }
    SN_KECCAK_CACHE.get(data).map(|v| *v)
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
pub fn sn_keccak_origin_get(value: Felt) -> Option<String> {
    if !*HASH_CACHE_ENABLED {
        return None;
    }
    SN_KECCAK_ORIGIN_CACHE.get(&value).map(|v| v.value().clone())
}

#[inline]
pub fn sn_keccak_origin_insert(value: Felt, data_hex: &str) {
    if !*HASH_CACHE_ENABLED {
        return;
    }
    if SN_KECCAK_ORIGIN_CACHE.len() >= SN_KECCAK_ORIGIN_CACHE_CAPACITY {
        SN_KECCAK_ORIGIN_CACHE.clear();
    }
    SN_KECCAK_ORIGIN_CACHE.insert(value, data_hex.to_string());
}

#[inline]
pub fn pedersen_pair_get(left: Felt, right: Felt) -> Option<Felt> {
    if *HASH_CALC_TOTALS_ENABLED {
        HASH_CALC_TOTALS.with(|stats| {
            let mut stats = stats.borrow_mut();
            stats.pedersen = stats.pedersen.saturating_add(1);
        });
        HASH_CALC_UNIQUES.with(|uniques| {
            uniques.borrow_mut().pedersen_pair.insert((left, right));
        });
    }
    if !*HASH_CACHE_ENABLED {
        return None;
    }
    PEDERSEN_PAIR_CACHE.get(&(left, right)).map(|v| *v)
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
    if *HASH_CALC_TOTALS_ENABLED {
        HASH_CALC_TOTALS.with(|stats| {
            let mut stats = stats.borrow_mut();
            stats.pedersen = stats.pedersen.saturating_add(1);
        });
        HASH_CALC_UNIQUES.with(|uniques| {
            uniques.borrow_mut().pedersen_array.insert(values.to_vec());
        });
    }
    if !*HASH_CACHE_ENABLED {
        return None;
    }
    PEDERSEN_ARRAY_CACHE.get(values).map(|v| *v)
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
    if *HASH_CALC_TOTALS_ENABLED {
        HASH_CALC_TOTALS.with(|stats| {
            let mut stats = stats.borrow_mut();
            stats.poseidon = stats.poseidon.saturating_add(1);
        });
        HASH_CALC_UNIQUES.with(|uniques| {
            uniques.borrow_mut().poseidon_array.insert(values.to_vec());
        });
    }
    if !*HASH_CACHE_ENABLED {
        return None;
    }
    POSEIDON_ARRAY_CACHE.get(values).map(|v| *v)
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
