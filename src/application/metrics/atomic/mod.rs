use std::sync::atomic::Ordering;

use prometheus_client::metrics::gauge;

pub struct AtomicU64(atomic_shim::AtomicU64);

impl gauge::Atomic<u64> for AtomicU64 {
    fn inc(&self) -> u64 {
        self.inc_by(1)
    }

    fn inc_by(&self, v: u64) -> u64 {
        self.0.fetch_add(v, Ordering::Relaxed)
    }

    fn dec(&self) -> u64 {
        self.dec_by(1)
    }

    fn dec_by(&self, v: u64) -> u64 {
        self.0.fetch_sub(v, Ordering::Relaxed)
    }

    fn set(&self, v: u64) -> u64 {
        self.0.swap(v, Ordering::Relaxed)
    }

    fn get(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}

impl Default for AtomicU64 {
    fn default() -> Self {
        Self(Default::default())
    }
}
