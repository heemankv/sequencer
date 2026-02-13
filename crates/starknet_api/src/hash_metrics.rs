use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[derive(Debug, Default, Clone, Copy)]
pub struct HashMetrics {
    pub sn_keccak_us: u64,
    pub sn_keccak_calls: u64,
    pub pedersen_us: u64,
    pub pedersen_calls: u64,
}

static HASH_TIMING_ENABLED: AtomicBool = AtomicBool::new(false);
static SN_KECCAK_US: AtomicU64 = AtomicU64::new(0);
static SN_KECCAK_CALLS: AtomicU64 = AtomicU64::new(0);
static PEDERSEN_US: AtomicU64 = AtomicU64::new(0);
static PEDERSEN_CALLS: AtomicU64 = AtomicU64::new(0);

#[inline]
pub fn start_hash_timing() {
    HASH_TIMING_ENABLED.store(true, Ordering::Relaxed);
    SN_KECCAK_US.store(0, Ordering::Relaxed);
    SN_KECCAK_CALLS.store(0, Ordering::Relaxed);
    PEDERSEN_US.store(0, Ordering::Relaxed);
    PEDERSEN_CALLS.store(0, Ordering::Relaxed);
}

#[inline]
pub fn stop_hash_timing() -> HashMetrics {
    let metrics = HashMetrics {
        sn_keccak_us: SN_KECCAK_US.swap(0, Ordering::Relaxed),
        sn_keccak_calls: SN_KECCAK_CALLS.swap(0, Ordering::Relaxed),
        pedersen_us: PEDERSEN_US.swap(0, Ordering::Relaxed),
        pedersen_calls: PEDERSEN_CALLS.swap(0, Ordering::Relaxed),
    };
    HASH_TIMING_ENABLED.store(false, Ordering::Relaxed);
    metrics
}

#[inline]
pub fn hash_timing_enabled() -> bool {
    HASH_TIMING_ENABLED.load(Ordering::Relaxed)
}

#[inline]
pub fn record_sn_keccak(us: u64) {
    if !hash_timing_enabled() {
        return;
    }
    SN_KECCAK_US.fetch_add(us, Ordering::Relaxed);
    SN_KECCAK_CALLS.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub fn record_pedersen(us: u64) {
    if !hash_timing_enabled() {
        return;
    }
    PEDERSEN_US.fetch_add(us, Ordering::Relaxed);
    PEDERSEN_CALLS.fetch_add(1, Ordering::Relaxed);
}
