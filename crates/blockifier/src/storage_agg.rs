use std::cell::RefCell;
use std::collections::HashSet;
use std::env;
use std::sync::OnceLock;

use starknet_api::core::ContractAddress;
use starknet_api::state::StorageKey;

#[derive(Default)]
struct StorageAggStats {
    cached_state_reads_total: u64,
    cached_state_cache_hits: u64,
    cached_state_cache_misses: u64,
    cached_state_read_keys: HashSet<(ContractAddress, StorageKey)>,
    cached_state_writes_total: u64,
    cached_state_write_keys: HashSet<(ContractAddress, StorageKey)>,
    cached_state_read_us: u64,
    cached_state_write_us: u64,
    layered_reads_total: u64,
    layered_cache_hits: u64,
    layered_cache_misses: u64,
    layered_read_keys: HashSet<(ContractAddress, StorageKey)>,
    layered_read_us: u64,
    backend_reads_total: u64,
    backend_read_keys: HashSet<(ContractAddress, StorageKey)>,
    backend_read_us: u64,
}

#[derive(Default, Clone, Copy)]
pub struct StorageAggSnapshot {
    pub cached_state_reads_total: u64,
    pub cached_state_cache_hits: u64,
    pub cached_state_cache_misses: u64,
    pub cached_state_unique_reads: u64,
    pub cached_state_writes_total: u64,
    pub cached_state_unique_writes: u64,
    pub cached_state_read_us: u64,
    pub cached_state_write_us: u64,
    pub layered_reads_total: u64,
    pub layered_cache_hits: u64,
    pub layered_cache_misses: u64,
    pub layered_unique_reads: u64,
    pub layered_read_us: u64,
    pub backend_reads_total: u64,
    pub backend_unique_reads: u64,
    pub backend_read_us: u64,
}

thread_local! {
    static STATS: RefCell<StorageAggStats> = RefCell::new(StorageAggStats::default());
}

static STORAGE_AGG_ENABLED: OnceLock<bool> = OnceLock::new();

#[inline]
pub fn enabled() -> bool {
    *STORAGE_AGG_ENABLED.get_or_init(|| {
        env::var("STORAGE_AGG_LOGS")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    })
}

#[inline]
pub fn reset() {
    if !enabled() {
        return;
    }
    STATS.with(|stats| *stats.borrow_mut() = StorageAggStats::default());
}

#[inline]
pub fn snapshot() -> StorageAggSnapshot {
    if !enabled() {
        return StorageAggSnapshot::default();
    }
    STATS.with(|stats| {
        let stats = stats.borrow();
        StorageAggSnapshot {
            cached_state_reads_total: stats.cached_state_reads_total,
            cached_state_cache_hits: stats.cached_state_cache_hits,
            cached_state_cache_misses: stats.cached_state_cache_misses,
            cached_state_unique_reads: stats.cached_state_read_keys.len() as u64,
            cached_state_writes_total: stats.cached_state_writes_total,
            cached_state_unique_writes: stats.cached_state_write_keys.len() as u64,
            cached_state_read_us: stats.cached_state_read_us,
            cached_state_write_us: stats.cached_state_write_us,
            layered_reads_total: stats.layered_reads_total,
            layered_cache_hits: stats.layered_cache_hits,
            layered_cache_misses: stats.layered_cache_misses,
            layered_unique_reads: stats.layered_read_keys.len() as u64,
            layered_read_us: stats.layered_read_us,
            backend_reads_total: stats.backend_reads_total,
            backend_unique_reads: stats.backend_read_keys.len() as u64,
            backend_read_us: stats.backend_read_us,
        }
    })
}

#[inline]
pub fn record_cached_state_read(
    contract_address: ContractAddress,
    key: StorageKey,
    hit: bool,
    elapsed_us: u64,
) {
    if !enabled() {
        return;
    }
    STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        stats.cached_state_reads_total = stats.cached_state_reads_total.saturating_add(1);
        if hit {
            stats.cached_state_cache_hits = stats.cached_state_cache_hits.saturating_add(1);
        } else {
            stats.cached_state_cache_misses = stats.cached_state_cache_misses.saturating_add(1);
        }
        stats.cached_state_read_us = stats.cached_state_read_us.saturating_add(elapsed_us);
        stats.cached_state_read_keys.insert((contract_address, key));
    });
}

#[inline]
pub fn record_cached_state_write(contract_address: ContractAddress, key: StorageKey, elapsed_us: u64) {
    if !enabled() {
        return;
    }
    STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        stats.cached_state_writes_total = stats.cached_state_writes_total.saturating_add(1);
        stats.cached_state_write_us = stats.cached_state_write_us.saturating_add(elapsed_us);
        stats.cached_state_write_keys.insert((contract_address, key));
    });
}

#[inline]
pub fn record_layered_read(contract_address: ContractAddress, key: StorageKey, hit: bool, elapsed_us: u64) {
    if !enabled() {
        return;
    }
    STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        stats.layered_reads_total = stats.layered_reads_total.saturating_add(1);
        if hit {
            stats.layered_cache_hits = stats.layered_cache_hits.saturating_add(1);
        } else {
            stats.layered_cache_misses = stats.layered_cache_misses.saturating_add(1);
        }
        stats.layered_read_us = stats.layered_read_us.saturating_add(elapsed_us);
        stats.layered_read_keys.insert((contract_address, key));
    });
}

#[inline]
pub fn record_backend_read(contract_address: ContractAddress, key: StorageKey, elapsed_us: u64) {
    if !enabled() {
        return;
    }
    STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        stats.backend_reads_total = stats.backend_reads_total.saturating_add(1);
        stats.backend_read_us = stats.backend_read_us.saturating_add(elapsed_us);
        stats.backend_read_keys.insert((contract_address, key));
    });
}
