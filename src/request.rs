use rand::{rngs::StdRng, seq::IndexedRandom, SeedableRng};
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;

use crate::rate_limiter::RateLimiter;

const ACCOUNTS: &'static [&'static str] = &include_lines!("accounts.txt");

pub async fn run(client: &Client, url: &str, rate_limiter: Arc<RateLimiter>) {
    const_assert!(ACCOUNTS.len() > 0);

    let mut rng = StdRng::from_os_rng(); // 更高效的 RNG

    loop {
        let query_account = ACCOUNTS.choose(&mut rng).unwrap();

        // 发送请求
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "cfx_getBalance",
            "params": [query_account],
            "id": 1
        });

        let success = match client.post(url).json(&request_body).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    response.text().await.is_ok()
                } else {
                    println!("Error status: {}", response.status());
                    false
                }
            }
            Err(e) => {
                println!("Request error: {}", e);
                false
            }
        };
        if !rate_limiter.next(success).await {
            break;
        }
    }
}
