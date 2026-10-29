pub mod info;
pub mod latency;
pub mod download;
pub mod upload;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

use crate::state::TestEvent;
use self::info::fetch_network_info;
use self::latency::{compute_stats, measure_latency_sample};
use self::download::run_download_test;
use self::upload::run_upload_test;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestPhaseToRun {
    All,
    LatencyOnly,
    DownloadOnly,
    UploadOnly,
}

pub async fn run_speed_test_pipeline(
    tx: UnboundedSender<TestEvent>,
    cancelled: Arc<AtomicBool>,
    phase: TestPhaseToRun,
    duration_per_phase: Duration,
    concurrency: usize,
) {
    let client = reqwest::Client::builder()
        .tcp_nodelay(true)
        .pool_max_idle_per_host(10)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    // 1. Network Info
    if phase == TestPhaseToRun::All || phase == TestPhaseToRun::LatencyOnly {
        let _ = tx.send(TestEvent::StatusUpdate("Detecting network and ISP information...".into()));
        match fetch_network_info(&client).await {
            Ok(info) => {
                let _ = tx.send(TestEvent::NetworkInfoDiscovered(info));
            }
            Err(e) => {
                let _ = tx.send(TestEvent::Log(format!("Could not fetch network info: {}", e)));
            }
        }
    }

    if cancelled.load(Ordering::Relaxed) {
        let _ = tx.send(TestEvent::Finished);
        return;
    }

    // 2. Latency & Jitter test
    if phase == TestPhaseToRun::All || phase == TestPhaseToRun::LatencyOnly {
        let _ = tx.send(TestEvent::StatusUpdate("Measuring Ping, Jitter & Packet Loss...".into()));
        let mut samples = Vec::new();
        let total_samples = 25;
        let ping_url = "https://speed.cloudflare.com/__down?bytes=0";

        for i in 1..=total_samples {
            if cancelled.load(Ordering::Relaxed) {
                break;
            }

            match measure_latency_sample(&client, ping_url).await {
                Ok(rtt) => {
                    samples.push(rtt);
                    let stats = compute_stats(&samples, i);
                    let _ = tx.send(TestEvent::LatencySample {
                        sample_ms: rtt,
                        stats,
                        progress: i as f64 / total_samples as f64,
                    });
                }
                Err(_) => {
                    let stats = compute_stats(&samples, i);
                    let _ = tx.send(TestEvent::LatencySample {
                        sample_ms: -1.0,
                        stats,
                        progress: i as f64 / total_samples as f64,
                    });
                }
            }

            tokio::time::sleep(Duration::from_millis(80)).await;
        }

        let final_stats = compute_stats(&samples, total_samples);
        let _ = tx.send(TestEvent::LatencyComplete(final_stats));
    }

    if cancelled.load(Ordering::Relaxed) {
        let _ = tx.send(TestEvent::Finished);
        return;
    }

    // 3. Download Speed Test
    if phase == TestPhaseToRun::All || phase == TestPhaseToRun::DownloadOnly {
        let _ = tx.send(TestEvent::StatusUpdate("Benchmarking Download Throughput...".into()));
        run_download_test(
            client.clone(),
            tx.clone(),
            duration_per_phase,
            concurrency,
            2_500_000, // 2.5 MB chunk
        )
        .await;
    }

    if cancelled.load(Ordering::Relaxed) {
        let _ = tx.send(TestEvent::Finished);
        return;
    }

    // 4. Upload Speed Test
    if phase == TestPhaseToRun::All || phase == TestPhaseToRun::UploadOnly {
        let _ = tx.send(TestEvent::StatusUpdate("Benchmarking Upload Throughput...".into()));
        run_upload_test(
            client.clone(),
            tx.clone(),
            duration_per_phase,
            concurrency,
            1_000_000, // 1 MB chunk
        )
        .await;
    }

    let _ = tx.send(TestEvent::StatusUpdate("Test sequence complete!".into()));
    let _ = tx.send(TestEvent::Finished);
}
