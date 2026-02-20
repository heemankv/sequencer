use starknet_api::transaction::TransactionHash;

use crate::storage_agg;

#[inline]
pub fn enabled() -> bool {
    storage_agg::enabled()
}

#[inline]
pub fn reset_for_tx() {
    if !enabled() {
        return;
    }
    storage_agg::reset();
}

#[inline]
pub fn log_for_tx(tx_hash: TransactionHash, outcome: &str) {
    if !enabled() {
        return;
    }
    let stats = storage_agg::snapshot();
    let total_unique_reads = stats
        .cached_state_unique_reads
        .saturating_add(stats.layered_unique_reads)
        .saturating_add(stats.backend_unique_reads);
    let total_unique_writes = stats.cached_state_unique_writes;

    log::info!(
        "blockifier_storage_total tx={} outcome={} cached_state_reads_total={} cached_state_cache_hits={} \
cached_state_cache_misses={} cached_state_read_us={} cached_state_writes_total={} cached_state_write_us={} \
layered_reads_total={} layered_cache_hits={} layered_cache_misses={} layered_read_us={} backend_reads_total={} \
backend_read_us={}",
        tx_hash,
        outcome,
        stats.cached_state_reads_total,
        stats.cached_state_cache_hits,
        stats.cached_state_cache_misses,
        stats.cached_state_read_us,
        stats.cached_state_writes_total,
        stats.cached_state_write_us,
        stats.layered_reads_total,
        stats.layered_cache_hits,
        stats.layered_cache_misses,
        stats.layered_read_us,
        stats.backend_reads_total,
        stats.backend_read_us,
    );

    log::info!(
        "blockifier_storage_unique tx={} cached_state_unique_reads={} cached_state_unique_writes={} \
layered_unique_reads={} backend_unique_reads={} total_unique_reads={} total_unique_writes={}",
        tx_hash,
        stats.cached_state_unique_reads,
        stats.cached_state_unique_writes,
        stats.layered_unique_reads,
        stats.backend_unique_reads,
        total_unique_reads,
        total_unique_writes,
    );
}
