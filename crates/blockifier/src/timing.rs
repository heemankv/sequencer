use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[derive(Debug, Default, Clone, Copy)]
pub struct BlockifierTiming {
    pub hash_pedersen_us: u64,
    pub hash_pedersen_calls: u64,
    pub hash_poseidon_us: u64,
    pub hash_poseidon_calls: u64,
    pub hash_sn_keccak_us: u64,
    pub hash_sn_keccak_calls: u64,
    pub hash_cairo_keccak_us: u64,
    pub hash_cairo_keccak_calls: u64,
    pub state_read_us: u64,
    pub state_read_calls: u64,
    pub state_write_us: u64,
    pub state_write_calls: u64,
}

static TIMING_ENABLED: AtomicBool = AtomicBool::new(false);
static STATE_READ_US: AtomicU64 = AtomicU64::new(0);
static STATE_READ_CALLS: AtomicU64 = AtomicU64::new(0);
static STATE_WRITE_US: AtomicU64 = AtomicU64::new(0);
static STATE_WRITE_CALLS: AtomicU64 = AtomicU64::new(0);
static CAIRO_KECCAK_US: AtomicU64 = AtomicU64::new(0);
static CAIRO_KECCAK_CALLS: AtomicU64 = AtomicU64::new(0);

#[inline]
pub fn timing_enabled() -> bool {
    TIMING_ENABLED.load(Ordering::Relaxed)
}

#[inline]
pub fn start_timing() {
    TIMING_ENABLED.store(true, Ordering::Relaxed);
    STATE_READ_US.store(0, Ordering::Relaxed);
    STATE_READ_CALLS.store(0, Ordering::Relaxed);
    STATE_WRITE_US.store(0, Ordering::Relaxed);
    STATE_WRITE_CALLS.store(0, Ordering::Relaxed);
    CAIRO_KECCAK_US.store(0, Ordering::Relaxed);
    CAIRO_KECCAK_CALLS.store(0, Ordering::Relaxed);

    #[cfg(feature = "cairo_native")]
    {
        cairo_native::runtime::start_hash_timing();
    }
    starknet_api::hash_metrics::start_hash_timing();
}

#[inline]
pub fn stop_timing() -> BlockifierTiming {
    let state_read_us = STATE_READ_US.swap(0, Ordering::Relaxed);
    let state_read_calls = STATE_READ_CALLS.swap(0, Ordering::Relaxed);
    let state_write_us = STATE_WRITE_US.swap(0, Ordering::Relaxed);
    let state_write_calls = STATE_WRITE_CALLS.swap(0, Ordering::Relaxed);
    let hash_cairo_keccak_us = CAIRO_KECCAK_US.swap(0, Ordering::Relaxed);
    let hash_cairo_keccak_calls = CAIRO_KECCAK_CALLS.swap(0, Ordering::Relaxed);

    let (mut hash_pedersen_us, mut hash_pedersen_calls, hash_poseidon_us, hash_poseidon_calls): (
        u64,
        u64,
        u64,
        u64,
    ) = {
        #[cfg(feature = "cairo_native")]
        {
            let native = cairo_native::runtime::stop_hash_timing_global();
            (
                native.pedersen_hash_us as u64,
                native.pedersen_calls,
                native.poseidon_hash_us as u64,
                native.poseidon_calls,
            )
        }
        #[cfg(not(feature = "cairo_native"))]
        {
            (0, 0, 0, 0)
        }
    };

    let sn = starknet_api::hash_metrics::stop_hash_timing();
    hash_pedersen_us = hash_pedersen_us.saturating_add(sn.pedersen_us);
    hash_pedersen_calls = hash_pedersen_calls.saturating_add(sn.pedersen_calls);

    TIMING_ENABLED.store(false, Ordering::Relaxed);

    BlockifierTiming {
        hash_pedersen_us,
        hash_pedersen_calls,
        hash_poseidon_us,
        hash_poseidon_calls,
        hash_sn_keccak_us: sn.sn_keccak_us,
        hash_sn_keccak_calls: sn.sn_keccak_calls,
        hash_cairo_keccak_us,
        hash_cairo_keccak_calls,
        state_read_us,
        state_read_calls,
        state_write_us,
        state_write_calls,
    }
}

#[inline]
pub fn record_state_read(us: u128) {
    if !timing_enabled() {
        return;
    }
    STATE_READ_US.fetch_add(us as u64, Ordering::Relaxed);
    STATE_READ_CALLS.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub fn record_state_write(us: u128) {
    if !timing_enabled() {
        return;
    }
    STATE_WRITE_US.fetch_add(us as u64, Ordering::Relaxed);
    STATE_WRITE_CALLS.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub fn record_cairo_keccak(us: u128) {
    if !timing_enabled() {
        return;
    }
    CAIRO_KECCAK_US.fetch_add(us as u64, Ordering::Relaxed);
    CAIRO_KECCAK_CALLS.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub fn timing_logs_enabled() -> bool {
    false
}
