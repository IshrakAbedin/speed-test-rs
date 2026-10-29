use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use bytes::Bytes;
use tokio::sync::mpsc::UnboundedSender;

use crate::state::TestEvent;

pub async fn run_upload_test(
    client: reqwest::Client,
    tx: UnboundedSender<TestEvent>,
    duration: Duration,
    concurrency: usize,
    chunk_bytes: usize,
) {
    let bytes_transferred = Arc::new(AtomicU64::new(0));
    let is_running = Arc::new(AtomicBool::new(true));

    // Pre-generate zero payload buffer to minimize CPU overhead
    let payload = Bytes::from(vec![0u8; chunk_bytes]);

    let mut handles = Vec::new();

    for _ in 0..concurrency {
        let client_clone = client.clone();
        let bytes_transferred_clone = Arc::clone(&bytes_transferred);
        let is_running_clone = Arc::clone(&is_running);
        let payload_clone = payload.clone();

        let handle = tokio::spawn(async move {
            let url = "https://speed.cloudflare.com/__up";
            while is_running_clone.load(Ordering::Relaxed) {
                let payload_len = payload_clone.len() as u64;
                match client_clone
                    .post(url)
                    .body(payload_clone.clone())
                    .send()
                    .await
                {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            bytes_transferred_clone.fetch_add(payload_len, Ordering::Relaxed);
                        }
                    }
                    Err(_) => {
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                }
            }
        });
        handles.push(handle);
    }

    let start_time = Instant::now();
    let mut last_instant = Instant::now();
    let mut last_bytes = 0u64;
    let mut peak_mbps = 0.0f64;

    while start_time.elapsed() < duration {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let now = Instant::now();
        let dt = (now - last_instant).as_secs_f64();
        if dt <= 0.0 {
            continue;
        }

        let current_total = bytes_transferred.load(Ordering::Relaxed);
        let delta_bytes = current_total.saturating_sub(last_bytes);
        let current_mbps = (delta_bytes as f64 * 8.0) / (dt * 1_000_000.0);
        if current_mbps > peak_mbps {
            peak_mbps = current_mbps;
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let avg_mbps = if elapsed > 0.0 {
            (current_total as f64 * 8.0) / (elapsed * 1_000_000.0)
        } else {
            0.0
        };

        let progress = (elapsed / duration.as_secs_f64()).min(1.0);

        let _ = tx.send(TestEvent::UploadProgress {
            current_mbps,
            average_mbps: avg_mbps,
            peak_mbps,
            bytes_transferred: current_total,
            progress,
        });

        last_instant = now;
        last_bytes = current_total;
    }

    is_running.store(false, Ordering::Relaxed);
    for h in handles {
        h.abort();
    }

    let total_bytes = bytes_transferred.load(Ordering::Relaxed);
    let total_secs = start_time.elapsed().as_secs_f64();
    let final_avg_mbps = if total_secs > 0.0 {
        (total_bytes as f64 * 8.0) / (total_secs * 1_000_000.0)
    } else {
        0.0
    };

    let _ = tx.send(TestEvent::UploadComplete {
        final_mbps: final_avg_mbps,
        peak_mbps,
        total_bytes,
    });
}
