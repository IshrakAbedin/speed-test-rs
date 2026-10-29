use std::time::Instant;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    pub min_ms: f64,
    pub max_ms: f64,
    pub avg_ms: f64,
    pub jitter_ms: f64,
    pub samples: Vec<f64>,
    pub packet_loss_pct: f64,
}

impl Default for LatencyStats {
    fn default() -> Self {
        Self {
            min_ms: 0.0,
            max_ms: 0.0,
            avg_ms: 0.0,
            jitter_ms: 0.0,
            samples: Vec::new(),
            packet_loss_pct: 0.0,
        }
    }
}

pub async fn measure_latency_sample(
    client: &reqwest::Client,
    url: &str,
) -> Result<f64, anyhow::Error> {
    let start = Instant::now();
    let resp = client
        .get(url)
        .timeout(std::time::Duration::from_millis(2500))
        .send()
        .await?;

    if resp.status().is_success() {
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        Ok(elapsed)
    } else {
        Err(anyhow::anyhow!("Non-200 status code: {}", resp.status()))
    }
}

/// Computes RFC 3550 style statistical jitter and min/avg/max
pub fn compute_stats(samples: &[f64], total_attempted: usize) -> LatencyStats {
    if samples.is_empty() {
        return LatencyStats {
            packet_loss_pct: if total_attempted > 0 { 100.0 } else { 0.0 },
            ..Default::default()
        };
    }

    let min_ms = samples.iter().copied().fold(f64::INFINITY, f64::min);
    let max_ms = samples.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let sum: f64 = samples.iter().sum();
    let avg_ms = sum / samples.len() as f64;

    // RFC 3550 Jitter calculation: D(i,j) = |(R_j - S_j) - (R_i - S_i)|
    // J = J + (|D| - J) / 16
    let mut jitter_ms = 0.0;
    if samples.len() > 1 {
        let mut prev = samples[0];
        for &curr in &samples[1..] {
            let diff = (curr - prev).abs();
            jitter_ms += (diff - jitter_ms) / 16.0;
            prev = curr;
        }
    }

    let failed = total_attempted.saturating_sub(samples.len());
    let packet_loss_pct = if total_attempted > 0 {
        (failed as f64 / total_attempted as f64) * 100.0
    } else {
        0.0
    };

    LatencyStats {
        min_ms,
        max_ms,
        avg_ms,
        jitter_ms,
        samples: samples.to_vec(),
        packet_loss_pct,
    }
}
