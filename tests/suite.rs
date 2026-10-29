use speed_test_rs::engine::info::NetworkInfo;
use speed_test_rs::engine::latency::{compute_stats, LatencyStats};
use speed_test_rs::state::{AppState, SpeedUnit, TelemetryState, TestEvent};

#[test]
fn test_speed_unit_toggle_and_formatting() {
    let unit = SpeedUnit::Mbps;
    assert_eq!(unit.toggle(), SpeedUnit::MBs);
    assert_eq!(unit.toggle().toggle(), SpeedUnit::Mbps);

    assert_eq!(SpeedUnit::Mbps.format_rate(100.0), "100.00 Mbps");
    // 100 Mbps / 8 = 12.50 MB/s
    assert_eq!(SpeedUnit::MBs.format_rate(100.0), "12.50 MB/s");
    assert_eq!(SpeedUnit::Mbps.format_rate(0.0), "0.00 Mbps");
    assert_eq!(SpeedUnit::MBs.format_rate(0.0), "0.00 MB/s");
}

#[test]
fn test_latency_stats_empty_samples() {
    let stats = compute_stats(&[], 0);
    assert_eq!(stats.min_ms, 0.0);
    assert_eq!(stats.max_ms, 0.0);
    assert_eq!(stats.avg_ms, 0.0);
    assert_eq!(stats.jitter_ms, 0.0);
    assert_eq!(stats.packet_loss_pct, 0.0);
    assert!(stats.samples.is_empty());

    // When attempted > 0 but no samples succeeded => 100% loss
    let stats_loss = compute_stats(&[], 5);
    assert_eq!(stats_loss.packet_loss_pct, 100.0);
}

#[test]
fn test_latency_stats_single_sample() {
    let samples = vec![25.0];
    let stats = compute_stats(&samples, 1);
    assert_eq!(stats.min_ms, 25.0);
    assert_eq!(stats.max_ms, 25.0);
    assert_eq!(stats.avg_ms, 25.0);
    assert_eq!(stats.jitter_ms, 0.0);
    assert_eq!(stats.packet_loss_pct, 0.0);
    assert_eq!(stats.samples, vec![25.0]);
}

#[test]
fn test_latency_and_rfc3550_jitter_calculation() {
    let samples = vec![10.0, 15.0, 12.0, 20.0];
    let stats = compute_stats(&samples, 4);

    assert_eq!(stats.min_ms, 10.0);
    assert_eq!(stats.max_ms, 20.0);
    assert_eq!(stats.avg_ms, (10.0 + 15.0 + 12.0 + 20.0) / 4.0);
    assert_eq!(stats.packet_loss_pct, 0.0);

    // RFC 3550 Jitter check:
    // sample 0: 10.0
    // sample 1: diff = |15 - 10| = 5.0, J = 0 + (5 - 0)/16 = 5/16 = 0.3125
    // sample 2: diff = |12 - 15| = 3.0, J = 0.3125 + (3 - 0.3125)/16 = 0.48046875
    // sample 3: diff = |20 - 12| = 8.0, J = 0.48046875 + (8 - 0.48046875)/16 = 0.950439453125
    assert!((stats.jitter_ms - 0.95043945).abs() < 1e-6);
}

#[test]
fn test_packet_loss_calculation() {
    let samples = vec![20.0, 22.0, 25.0];
    // 3 received out of 5 attempted = 40% loss
    let stats = compute_stats(&samples, 5);
    assert_eq!(stats.packet_loss_pct, 40.0);
}

#[test]
fn test_telemetry_state_lifecycle() {
    let mut state = TelemetryState::new();
    assert_eq!(state.status, AppState::Idle);
    assert_eq!(state.unit, SpeedUnit::Mbps);
    assert_eq!(state.download_avg_mbps, 0.0);
    assert_eq!(state.upload_avg_mbps, 0.0);

    // Handle NetworkInfo event
    let info = NetworkInfo {
        ip: "1.1.1.1".to_string(),
        city: "San Francisco".to_string(),
        region: "California".to_string(),
        country: "US".to_string(),
        loc: "37.77,-122.41".to_string(),
        org: "Cloudflare, Inc.".to_string(),
        timezone: "America/Los_Angeles".to_string(),
        colo: Some("SFO".to_string()),
    };
    state.handle_event(TestEvent::NetworkInfoDiscovered(info.clone()));
    assert_eq!(state.network_info.ip, "1.1.1.1");
    assert_eq!(state.network_info.colo.as_deref(), Some("SFO"));

    // Handle Latency sample & completion
    let lat_stats = LatencyStats {
        min_ms: 12.0,
        max_ms: 18.0,
        avg_ms: 15.0,
        jitter_ms: 1.2,
        samples: vec![12.0, 15.0, 18.0],
        packet_loss_pct: 0.0,
    };
    state.handle_event(TestEvent::LatencyComplete(lat_stats.clone()));
    assert_eq!(state.ping_stats.avg_ms, 15.0);
    assert_eq!(state.ping_stats.jitter_ms, 1.2);
    assert_eq!(state.ping_progress, 1.0);

    // Handle Download events
    state.handle_event(TestEvent::DownloadProgress {
        current_mbps: 120.5,
        average_mbps: 110.0,
        peak_mbps: 125.0,
        bytes_transferred: 50_000_000,
        progress: 0.5,
    });
    assert_eq!(state.download_current_mbps, 120.5);
    assert_eq!(state.download_history.len(), 1);
    assert_eq!(state.download_history[0], 1205); // 120.5 * 10

    state.handle_event(TestEvent::DownloadComplete {
        final_mbps: 115.0,
        peak_mbps: 125.0,
        total_bytes: 100_000_000,
    });
    assert_eq!(state.download_avg_mbps, 115.0);
    assert_eq!(state.download_peak_mbps, 125.0);
    assert_eq!(state.download_progress, 1.0);

    // Handle Upload events
    state.handle_event(TestEvent::UploadProgress {
        current_mbps: 45.0,
        average_mbps: 40.0,
        peak_mbps: 50.0,
        bytes_transferred: 10_000_000,
        progress: 0.5,
    });
    assert_eq!(state.upload_current_mbps, 45.0);

    state.handle_event(TestEvent::UploadComplete {
        final_mbps: 42.0,
        peak_mbps: 50.0,
        total_bytes: 20_000_000,
    });
    assert_eq!(state.upload_avg_mbps, 42.0);
    assert_eq!(state.upload_peak_mbps, 50.0);
    assert_eq!(state.upload_progress, 1.0);

    // Handle Finished
    state.handle_event(TestEvent::Finished);
    assert_eq!(state.status, AppState::Finished);

    // Reset metrics test
    state.reset_metrics();
    assert_eq!(state.download_avg_mbps, 0.0);
    assert_eq!(state.upload_avg_mbps, 0.0);
    assert_eq!(state.download_history.len(), 0);
    assert_eq!(state.ping_stats.avg_ms, 0.0);
}

#[test]
fn test_json_report_export() {
    let mut state = TelemetryState::new();
    state.network_info.ip = "203.0.113.1".to_string();
    state.network_info.org = "Test ISP".to_string();
    state.ping_stats.avg_ms = 22.5;
    state.download_avg_mbps = 95.4;
    state.upload_avg_mbps = 33.1;

    let temp_file = std::env::temp_dir().join(format!("test_speedtest_export_{}.json", rand::random::<u32>()));
    let temp_path = temp_file.to_str().unwrap();

    let export_res = state.export_to_json(temp_path);
    assert!(export_res.is_ok());

    let file_content = std::fs::read_to_string(temp_path).expect("failed to read exported file");
    assert!(file_content.contains("\"ip\": \"203.0.113.1\""));
    assert!(file_content.contains("\"download_avg_mbps\": 95.4"));
    assert!(file_content.contains("\"upload_avg_mbps\": 33.1"));

    let _ = std::fs::remove_file(temp_path);
}
