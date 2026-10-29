use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::engine::info::NetworkInfo;
use crate::engine::latency::LatencyStats;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeedUnit {
    Mbps, // Megabits per second
    MBs,  // Megabytes per second
}

impl SpeedUnit {
    pub fn toggle(&self) -> Self {
        match self {
            SpeedUnit::Mbps => SpeedUnit::MBs,
            SpeedUnit::MBs => SpeedUnit::Mbps,
        }
    }

    pub fn format_rate(&self, mbps: f64) -> String {
        match self {
            SpeedUnit::Mbps => format!("{:.2} Mbps", mbps),
            SpeedUnit::MBs => format!("{:.2} MB/s", mbps / 8.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    Idle,
    Running,
    Finished,
}

#[derive(Debug, Clone)]
pub enum TestEvent {
    StatusUpdate(String),
    NetworkInfoDiscovered(NetworkInfo),
    LatencySample {
        sample_ms: f64,
        stats: LatencyStats,
        progress: f64,
    },
    LatencyComplete(LatencyStats),
    DownloadProgress {
        current_mbps: f64,
        average_mbps: f64,
        peak_mbps: f64,
        bytes_transferred: u64,
        progress: f64,
    },
    DownloadComplete {
        final_mbps: f64,
        peak_mbps: f64,
        total_bytes: u64,
    },
    UploadProgress {
        current_mbps: f64,
        average_mbps: f64,
        peak_mbps: f64,
        bytes_transferred: u64,
        progress: f64,
    },
    UploadComplete {
        final_mbps: f64,
        peak_mbps: f64,
        total_bytes: u64,
    },
    Log(String),
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportReport {
    pub timestamp: String,
    pub network_info: NetworkInfo,
    pub ping_min_ms: f64,
    pub ping_avg_ms: f64,
    pub ping_max_ms: f64,
    pub jitter_ms: f64,
    pub packet_loss_pct: f64,
    pub download_avg_mbps: f64,
    pub download_peak_mbps: f64,
    pub upload_avg_mbps: f64,
    pub upload_peak_mbps: f64,
    pub total_download_bytes: u64,
    pub total_upload_bytes: u64,
}

pub struct TelemetryState {
    pub status: AppState,
    pub status_message: String,
    pub unit: SpeedUnit,
    pub network_info: NetworkInfo,

    // Latency & Jitter
    pub ping_samples: Vec<f64>,
    pub ping_stats: LatencyStats,
    pub ping_progress: f64,

    // Download
    pub download_current_mbps: f64,
    pub download_avg_mbps: f64,
    pub download_peak_mbps: f64,
    pub download_history: Vec<u64>, // Scaled for sparkline
    pub download_points: Vec<f64>,  // Raw f64 Mbps for Braille Chart
    pub download_bytes: u64,
    pub download_progress: f64,

    // Upload
    pub upload_current_mbps: f64,
    pub upload_avg_mbps: f64,
    pub upload_peak_mbps: f64,
    pub upload_history: Vec<u64>, // Scaled for sparkline
    pub upload_points: Vec<f64>,  // Raw f64 Mbps for Braille Chart
    pub upload_bytes: u64,
    pub upload_progress: f64,

    // Logs & Notifications
    pub logs: Vec<String>,
    pub export_notification: Option<String>,
}

impl TelemetryState {
    pub fn new() -> Self {
        Self {
            status: AppState::Idle,
            status_message: "Ready. Press [Space] or [Enter] to start test.".to_string(),
            unit: SpeedUnit::Mbps,
            network_info: NetworkInfo::default(),

            ping_samples: Vec::new(),
            ping_stats: LatencyStats::default(),
            ping_progress: 0.0,

            download_current_mbps: 0.0,
            download_avg_mbps: 0.0,
            download_peak_mbps: 0.0,
            download_history: Vec::with_capacity(120),
            download_points: Vec::with_capacity(120),
            download_bytes: 0,
            download_progress: 0.0,

            upload_current_mbps: 0.0,
            upload_avg_mbps: 0.0,
            upload_peak_mbps: 0.0,
            upload_history: Vec::with_capacity(120),
            upload_points: Vec::with_capacity(120),
            upload_bytes: 0,
            upload_progress: 0.0,

            logs: Vec::new(),
            export_notification: None,
        }
    }

    pub fn reset_metrics(&mut self) {
        self.ping_samples.clear();
        self.ping_stats = LatencyStats::default();
        self.ping_progress = 0.0;

        self.download_current_mbps = 0.0;
        self.download_avg_mbps = 0.0;
        self.download_peak_mbps = 0.0;
        self.download_history.clear();
        self.download_points.clear();
        self.download_bytes = 0;
        self.download_progress = 0.0;

        self.upload_current_mbps = 0.0;
        self.upload_avg_mbps = 0.0;
        self.upload_peak_mbps = 0.0;
        self.upload_history.clear();
        self.upload_points.clear();
        self.upload_bytes = 0;
        self.upload_progress = 0.0;

        self.export_notification = None;
    }

    pub fn handle_event(&mut self, event: TestEvent) {
        match event {
            TestEvent::StatusUpdate(msg) => {
                self.status_message = msg.clone();
                self.add_log(msg);
            }
            TestEvent::NetworkInfoDiscovered(info) => {
                self.network_info = info;
                self.add_log(format!(
                    "Discovered IP: {} ({}, {}) Colo: {:?}",
                    self.network_info.ip, self.network_info.org, self.network_info.country, self.network_info.colo
                ));
            }
            TestEvent::LatencySample {
                sample_ms,
                stats,
                progress,
            } => {
                if sample_ms >= 0.0 {
                    self.ping_samples.push(sample_ms);
                    if self.ping_samples.len() > 100 {
                        self.ping_samples.remove(0);
                    }
                }
                self.ping_stats = stats;
                self.ping_progress = progress;
            }
            TestEvent::LatencyComplete(stats) => {
                self.ping_stats = stats;
                self.ping_progress = 1.0;
                self.add_log(format!(
                    "Ping test complete: avg {:.1}ms, jitter {:.1}ms, loss {:.1}%",
                    self.ping_stats.avg_ms, self.ping_stats.jitter_ms, self.ping_stats.packet_loss_pct
                ));
            }
            TestEvent::DownloadProgress {
                current_mbps,
                average_mbps,
                peak_mbps,
                bytes_transferred,
                progress,
            } => {
                self.download_current_mbps = current_mbps;
                self.download_avg_mbps = average_mbps;
                self.download_peak_mbps = peak_mbps;
                self.download_bytes = bytes_transferred;
                self.download_progress = progress;

                // Push rounded Mbps to history for block fallback
                self.download_history.push((current_mbps * 10.0) as u64);
                if self.download_history.len() > 80 {
                    self.download_history.remove(0);
                }

                // Push raw f64 for high-resolution Braille chart
                self.download_points.push(current_mbps);
                if self.download_points.len() > 100 {
                    self.download_points.remove(0);
                }
            }
            TestEvent::DownloadComplete {
                final_mbps,
                peak_mbps,
                total_bytes,
            } => {
                self.download_avg_mbps = final_mbps;
                self.download_peak_mbps = peak_mbps;
                self.download_bytes = total_bytes;
                self.download_progress = 1.0;
                self.add_log(format!(
                    "Download test complete: avg {:.2} Mbps (peak {:.2} Mbps)",
                    final_mbps, peak_mbps
                ));
            }
            TestEvent::UploadProgress {
                current_mbps,
                average_mbps,
                peak_mbps,
                bytes_transferred,
                progress,
            } => {
                self.upload_current_mbps = current_mbps;
                self.upload_avg_mbps = average_mbps;
                self.upload_peak_mbps = peak_mbps;
                self.upload_bytes = bytes_transferred;
                self.upload_progress = progress;

                self.upload_history.push((current_mbps * 10.0) as u64);
                if self.upload_history.len() > 80 {
                    self.upload_history.remove(0);
                }

                self.upload_points.push(current_mbps);
                if self.upload_points.len() > 100 {
                    self.upload_points.remove(0);
                }
            }
            TestEvent::UploadComplete {
                final_mbps,
                peak_mbps,
                total_bytes,
            } => {
                self.upload_avg_mbps = final_mbps;
                self.upload_peak_mbps = peak_mbps;
                self.upload_bytes = total_bytes;
                self.upload_progress = 1.0;
                self.add_log(format!(
                    "Upload test complete: avg {:.2} Mbps (peak {:.2} Mbps)",
                    final_mbps, peak_mbps
                ));
            }
            TestEvent::Log(msg) => {
                self.add_log(msg);
            }
            TestEvent::Finished => {
                self.status = AppState::Finished;
                self.status_message = "All benchmarks finished! Press [R] to re-run, [E] to export, [Q] to quit.".into();
            }
        }
    }

    pub fn add_log(&mut self, log: String) {
        let timestamp = Local::now().format("%H:%M:%S").to_string();
        self.logs.push(format!("[{}] {}", timestamp, log));
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    pub fn export_to_json(&mut self, path: &str) -> Result<(), anyhow::Error> {
        let report = ExportReport {
            timestamp: Local::now().to_rfc3339(),
            network_info: self.network_info.clone(),
            ping_min_ms: self.ping_stats.min_ms,
            ping_avg_ms: self.ping_stats.avg_ms,
            ping_max_ms: self.ping_stats.max_ms,
            jitter_ms: self.ping_stats.jitter_ms,
            packet_loss_pct: self.ping_stats.packet_loss_pct,
            download_avg_mbps: self.download_avg_mbps,
            download_peak_mbps: self.download_peak_mbps,
            upload_avg_mbps: self.upload_avg_mbps,
            upload_peak_mbps: self.upload_peak_mbps,
            total_download_bytes: self.download_bytes,
            total_upload_bytes: self.upload_bytes,
        };

        let file = std::fs::File::create(path)?;
        serde_json::to_writer_pretty(file, &report)?;
        let msg = format!("Exported report to {}", path);
        self.export_notification = Some(msg.clone());
        self.add_log(msg);
        Ok(())
    }
}
