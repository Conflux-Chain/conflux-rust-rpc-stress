#[macro_use]
extern crate include_lines;
#[macro_use]
extern crate assertions;

mod opts;
mod rate_limiter;
mod request;

use rate_limiter::RateLimiter;
use reqwest::Client;
use std::{
    sync::{atomic::Ordering::Relaxed, Arc},
    time::{Duration, Instant},
};
use structopt::StructOpt;
use tokio::{select, task::JoinSet, time::sleep};

#[tokio::main]
async fn main() {
    let opt = opts::Opt::from_args();

    println!("Starting JSON-RPC client with {} threads", opt.thread_count);
    println!("Sending requests to {}", opt.rpc_url);

    let rate_limiter =
        Arc::new(RateLimiter::new(opt.target_qps, opt.target_count));

    let report_handle = {
        let rate_limiter = rate_limiter.clone();
        tokio::spawn(async move {
            let counter = &rate_limiter.success_requests;
            let report_interval = Duration::from_secs(opt.report_interval);

            let mut instant = Instant::now();
            let mut last_count = counter.load(Relaxed);

            loop {
                sleep(report_interval).await;
                let elapsed = instant.elapsed();
                let current_count = counter.load(Relaxed);
                let qps =
                    (current_count - last_count) as f64 / elapsed.as_secs_f64();
                println!(
                    "QPS: {:.2}, Total: {}, Elapsed: {:.2}s",
                    qps,
                    current_count,
                    elapsed.as_secs_f64(),
                );

                instant = Instant::now();
                last_count = counter.load(Relaxed);
            }
        })
    };

    let replenisher_handle = {
        let rate_limiter = rate_limiter.clone();
        tokio::spawn(async move { rate_limiter.replenish_permits().await })
    };

    // 创建工作线程
    let mut set = JoinSet::new();

    let client = Arc::new(Client::new());
    for _ in 0..opt.thread_count {
        let client = client.clone();
        let url = opt.rpc_url.clone();
        let rate_limiter = rate_limiter.clone();

        set.spawn(
            async move { request::run(&*client, &*url, rate_limiter).await },
        );
    }

    join_requesters_or_timeout(set, opt.max_time).await;

    let _ = report_handle.abort();
    let _ = replenisher_handle.abort();
}

async fn join_requesters_or_timeout<T: 'static>(
    mut set: JoinSet<T>, max_time: u64,
) {
    let timeout = sleep(Duration::from_secs(max_time));
    tokio::pin!(timeout);
    loop {
        select! {
            _ = &mut timeout => {
                set.abort_all();
                return;
            },
            res = set.join_next() => {
                if res.is_none() {
                    return;
                }
            }
        }
    }
}
