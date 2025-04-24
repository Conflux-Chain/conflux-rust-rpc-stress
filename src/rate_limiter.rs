use std::{
    sync::atomic::{
        AtomicU64,
        Ordering::{AcqRel, Acquire},
    },
    time::{Duration, Instant},
};
use tokio::{sync::Notify, time::sleep};

pub struct RateLimiter {
    start: Instant,
    qps: u64,
    pub success_requests: AtomicU64,
    target_requests: u64,
    notify: Notify,
}

impl RateLimiter {
    pub fn new(qps: u64, target_requests: u64) -> Self {
        Self {
            start: Instant::now(),
            success_requests: AtomicU64::new(0),
            target_requests,
            qps,
            notify: Notify::new(),
        }
    }

    pub async fn next(&self, success: bool) -> bool {
        let success_requests = if success {
            self.success_requests.fetch_add(1, AcqRel)
        } else {
            self.success_requests.load(Acquire)
        };

        let should_continue = success_requests < self.target_requests;

        if self.qps == 0 {
            return should_continue;
        }

        if success_requests as f64
            >= self.start.elapsed().as_secs_f64() * self.qps as f64
        {
            self.notify.notified().await;
        }

        self.success_requests.load(Acquire) < self.target_requests
    }

    pub async fn replenish_permits(&self) {
        if self.qps == 0 {
            return;
        }

        let interval = Duration::from_secs(1) / self.qps as u32;
        loop {
            sleep(interval).await;
            self.notify.notify_one()
        }
    }
}
