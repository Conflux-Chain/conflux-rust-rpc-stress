use reqwest::Client;
use serde_json::json;
use std::sync::Arc;

use crate::rate_limiter::RateLimiter;

pub async fn run(client: &Client, url: &str, rate_limiter: Arc<RateLimiter>) {
    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "cfx_getBalance",
        "params": ["net10:aar8jzybzv0fhzreav49syxnzut8s0jt1asmxx99xh"],
        "id": 1
    });

    loop {
        // 发送请求
        let success = match client.post(url).json(&request_body).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    true
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
