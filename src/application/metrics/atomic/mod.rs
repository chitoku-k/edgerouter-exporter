use std::sync::atomic::Ordering;

use prometheus_client::metrics::gauge;

#[derive(Debug, Default)]
pub struct AtomicU64(atomic_shim::AtomicU64);

#[derive(Debug, Default)]
pub struct AtomicI64(atomic_shim::AtomicI64);

impl gauge::Atomic<f64> for AtomicU64 {
    fn inc(&self) -> f64 {
        self.inc_by(1.0)
    }

    fn inc_by(&self, v: f64) -> f64 {
        let mut old_u64 = self.0.load(Ordering::Relaxed);
        let mut old_f64;
        loop {
            old_f64 = f64::from_bits(old_u64);
            let new = f64::to_bits(old_f64 + v);
            match self.0.compare_exchange_weak(old_u64, new, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => break,
                Err(x) => old_u64 = x,
            }
        }

        old_f64
    }

    fn dec(&self) -> f64 {
        self.dec_by(1.0)
    }

    fn dec_by(&self, v: f64) -> f64 {
        let mut old_u64 = self.0.load(Ordering::Relaxed);
        let mut old_f64;
        loop {
            old_f64 = f64::from_bits(old_u64);
            let new = f64::to_bits(old_f64 - v);
            match self.0.compare_exchange_weak(old_u64, new, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => break,
                Err(x) => old_u64 = x,
            }
        }

        old_f64
    }

    fn set(&self, v: f64) -> f64 {
        f64::from_bits(self.0.swap(f64::to_bits(v), Ordering::Relaxed))
    }

    fn get(&self) -> f64 {
        f64::from_bits(self.0.load(Ordering::Relaxed))
    }
}

impl gauge::Atomic<i64> for AtomicI64 {
    fn inc(&self) -> i64 {
        self.inc_by(1)
    }

    fn inc_by(&self, v: i64) -> i64 {
        self.0.fetch_add(v, Ordering::Relaxed)
    }

    fn dec(&self) -> i64 {
        self.dec_by(1)
    }

    fn dec_by(&self, v: i64) -> i64 {
        self.0.fetch_sub(v, Ordering::Relaxed)
    }

    fn set(&self, v: i64) -> i64 {
        self.0.swap(v, Ordering::Relaxed)
    }

    fn get(&self) -> i64 {
        self.0.load(Ordering::Relaxed)
    }
}
