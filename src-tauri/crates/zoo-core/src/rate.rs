//! Global bandwidth limit shared by every transfer.
//!
//! A token bucket that may go into debt: `acquire(n)` always takes `n` immediately and,
//! if that overdrew the bucket, sleeps until the debt is repaid. Large chunks therefore
//! never deadlock, and the long-run rate stays at the limit however many callers share it.

use std::sync::Mutex;
use tokio::time::{sleep, Duration, Instant};

pub struct RateLimiter {
    state: Mutex<Bucket>,
}

struct Bucket {
    /// Bytes per second; `None` means unlimited.
    rate: Option<u64>,
    tokens: f64,
    last: Instant,
}

impl RateLimiter {
    pub fn new(bytes_per_sec: Option<u64>) -> Self {
        let rate = bytes_per_sec.filter(|&r| r > 0);
        Self {
            state: Mutex::new(Bucket { rate, tokens: rate.unwrap_or(0) as f64, last: Instant::now() }),
        }
    }

    /// Change the limit; takes effect for the next `acquire`.
    pub fn set_rate(&self, bytes_per_sec: Option<u64>) {
        let mut b = self.state.lock().unwrap();
        b.rate = bytes_per_sec.filter(|&r| r > 0);
        b.tokens = b.tokens.min(b.rate.unwrap_or(0) as f64);
        b.last = Instant::now();
    }

    pub async fn acquire(&self, bytes: u64) {
        let wait = {
            let mut b = self.state.lock().unwrap();
            let Some(rate) = b.rate else { return };
            let now = Instant::now();
            let refill = now.duration_since(b.last).as_secs_f64() * rate as f64;
            // Burst capacity is one second's worth.
            b.tokens = (b.tokens + refill).min(rate as f64) - bytes as f64;
            b.last = now;
            if b.tokens >= 0.0 {
                return;
            }
            Duration::from_secs_f64(-b.tokens / rate as f64)
        };
        sleep(wait).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test(start_paused = true)]
    async fn unlimited_never_waits() {
        let limiter = RateLimiter::new(None);
        let start = Instant::now();
        limiter.acquire(u64::MAX / 2).await;
        assert_eq!(start.elapsed(), Duration::ZERO);
    }

    #[tokio::test(start_paused = true)]
    async fn long_run_rate_matches_limit() {
        let limiter = RateLimiter::new(Some(1000));
        let start = Instant::now();
        // 1s of burst is free, the remaining 4000 bytes take 4s.
        for _ in 0..50 {
            limiter.acquire(100).await;
        }
        let secs = start.elapsed().as_secs_f64();
        assert!((3.9..=4.1).contains(&secs), "took {secs}s");
    }

    #[tokio::test(start_paused = true)]
    async fn chunks_larger_than_capacity_do_not_deadlock() {
        let limiter = RateLimiter::new(Some(1000));
        let start = Instant::now();
        limiter.acquire(5000).await; // 1000 in the bucket, 4000 debt
        let secs = start.elapsed().as_secs_f64();
        assert!((3.9..=4.1).contains(&secs), "took {secs}s");
    }

    #[tokio::test(start_paused = true)]
    async fn rate_is_shared_across_concurrent_callers() {
        let limiter = Arc::new(RateLimiter::new(Some(1000)));
        let start = Instant::now();
        let tasks: Vec<_> = (0..4)
            .map(|_| {
                let l = limiter.clone();
                tokio::spawn(async move {
                    for _ in 0..10 {
                        l.acquire(125).await;
                    }
                })
            })
            .collect();
        for t in tasks {
            t.await.unwrap();
        }
        // 5000 bytes total: 1000 burst + 4000 at 1000/s.
        let secs = start.elapsed().as_secs_f64();
        assert!((3.5..=4.5).contains(&secs), "took {secs}s");
    }

    #[tokio::test(start_paused = true)]
    async fn set_rate_applies_immediately() {
        let limiter = RateLimiter::new(Some(100));
        limiter.set_rate(None);
        let start = Instant::now();
        limiter.acquire(1_000_000).await;
        assert_eq!(start.elapsed(), Duration::ZERO);
    }
}
