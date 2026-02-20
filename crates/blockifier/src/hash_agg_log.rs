use crate::state::state_api::StateReader;
use starknet_api::transaction::TransactionHash;

#[derive(Default, Clone, Copy)]
struct HashAggTotals {
    pedersen_calls: u64,
    pedersen_hits: u64,
    pedersen_misses: u64,
    pedersen_inputs: u64,
    poseidon_calls: u64,
    poseidon_inputs: u64,
    sn_keccak_calls: u64,
    sn_keccak_hits: u64,
    sn_keccak_misses: u64,
    sn_keccak_inputs: u64,
}

#[inline]
pub fn enabled() -> bool {
    starknet_api::hash_agg::enabled()
}

#[inline]
pub fn reset_for_tx<S: StateReader>(state: &S) {
    if !enabled() {
        return;
    }
    starknet_api::hash_agg::reset();
    state.reset_cache_stats();
    #[cfg(feature = "cairo_native")]
    cairo_native::hash_agg::reset();
}

#[inline]
pub fn log_for_tx<S: StateReader>(tx_hash: TransactionHash, outcome: &str, state: &S) {
    if !enabled() {
        return;
    }
    let hash_stats = combined_hash_stats();
    let cache_stats = state.cache_stats_snapshot();
    let total_unique = hash_stats
        .pedersen_inputs
        .saturating_add(hash_stats.poseidon_inputs)
        .saturating_add(hash_stats.sn_keccak_inputs);

    log::info!(
        "blockifier_hash_total tx={} outcome={} pedersen_calls={} pedersen_hits={} \
         pedersen_misses={} poseidon_calls={} sn_keccak_calls={} sn_keccak_hits={} \
         sn_keccak_misses={} cached_state_reads_total={} cached_state_cache_hits={} \
         cached_state_cache_misses={}",
        tx_hash,
        outcome,
        hash_stats.pedersen_calls,
        hash_stats.pedersen_hits,
        hash_stats.pedersen_misses,
        hash_stats.poseidon_calls,
        hash_stats.sn_keccak_calls,
        hash_stats.sn_keccak_hits,
        hash_stats.sn_keccak_misses,
        cache_stats.reads_total,
        cache_stats.cache_hits,
        cache_stats.cache_misses,
    );

    log::info!(
        "blockifier_hash_unique tx={} pedersen_inputs={} poseidon_inputs={} sn_keccak_inputs={} \
         total_unique={}",
        tx_hash,
        hash_stats.pedersen_inputs,
        hash_stats.poseidon_inputs,
        hash_stats.sn_keccak_inputs,
        total_unique,
    );
}

fn combined_hash_stats() -> HashAggTotals {
    let api = starknet_api::hash_agg::snapshot();
    let mut totals = HashAggTotals {
        pedersen_calls: api.pedersen_calls,
        pedersen_hits: api.pedersen_hits,
        pedersen_misses: api.pedersen_misses,
        pedersen_inputs: api.pedersen_inputs,
        poseidon_calls: api.poseidon_calls,
        poseidon_inputs: api.poseidon_inputs,
        sn_keccak_calls: api.sn_keccak_calls,
        sn_keccak_hits: api.sn_keccak_hits,
        sn_keccak_misses: api.sn_keccak_misses,
        sn_keccak_inputs: api.sn_keccak_inputs,
    };

    #[cfg(feature = "cairo_native")]
    {
        let native = cairo_native::hash_agg::snapshot();
        totals.pedersen_calls = totals.pedersen_calls.saturating_add(native.pedersen_calls);
        totals.pedersen_hits = totals.pedersen_hits.saturating_add(native.pedersen_hits);
        totals.pedersen_misses = totals.pedersen_misses.saturating_add(native.pedersen_misses);
        totals.pedersen_inputs = totals.pedersen_inputs.saturating_add(native.pedersen_inputs);
        totals.poseidon_calls = totals.poseidon_calls.saturating_add(native.poseidon_calls);
        totals.poseidon_inputs = totals.poseidon_inputs.saturating_add(native.poseidon_inputs);
        totals.sn_keccak_calls = totals.sn_keccak_calls.saturating_add(native.sn_keccak_calls);
        totals.sn_keccak_hits = totals.sn_keccak_hits.saturating_add(native.sn_keccak_hits);
        totals.sn_keccak_misses = totals.sn_keccak_misses.saturating_add(native.sn_keccak_misses);
        totals.sn_keccak_inputs = totals.sn_keccak_inputs.saturating_add(native.sn_keccak_inputs);
    }

    totals
}
