use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        canvas::{Canvas, Line as CanvasLine},
        Block, BorderType, Borders, Gauge, Paragraph, Row, Sparkline, Table, Wrap,
    },
    Frame,
};

use crate::app::App;
use crate::state::{AppState, SpeedUnit};
use crate::ui::theme::THEME;

pub fn render_ui(f: &mut Frame, app: &App) {
    let area = f.area();

    // Check if terminal height is compact
    let is_compact_height = area.height < 32;
    let is_very_compact = area.height < 24;

    // Allocate 4 rows for the header to neatly hold 2 lines (App title + clean telemetry details)
    let header_height = if area.width < 75 { 4 } else { 3 };

    let chunks = if is_very_compact {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(header_height), // Header
                Constraint::Length(4),             // Compact Latency
                Constraint::Length(5),             // Bandwidth
                Constraint::Min(4),                // Table or logs
                Constraint::Length(3),             // Centered Footer
            ])
            .split(area)
    } else if is_compact_height {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(header_height), // Header
                Constraint::Length(5),             // Latency row
                Constraint::Length(6),             // Download & Upload speed sparklines
                Constraint::Min(6),                // Table & Logs
                Constraint::Length(3),             // Centered Footer
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(header_height), // Header
                Constraint::Length(5),             // Latency, Jitter, Loss gauges
                Constraint::Length(7),             // Download & Upload real-time speed sparklines
                Constraint::Min(8),                // Metrics summary table & Event Logs
                Constraint::Length(3),             // Centered Footer
            ])
            .split(area)
    };

    render_header(f, app, chunks[0]);
    render_latency_row(f, app, chunks[1]);
    render_bandwidth_row(f, app, chunks[2]);
    render_details_and_logs(f, app, chunks[3]);
    render_footer(f, app, chunks[4]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let info = &app.telemetry.network_info;
    let colo_str = info.colo.as_deref().unwrap_or("Detecting...");

    let ip_str = if info.ip == "Unknown" {
        "Detecting...".to_string()
    } else {
        info.ip.clone()
    };

    let loc_str = if info.city == "Unknown" {
        "Detecting...".to_string()
    } else {
        format!("{}, {}", info.city, info.country)
    };

    let org_str = if info.org == "Unknown" {
        "Detecting...".to_string()
    } else {
        info.org.clone()
    };

    // Clean, adaptive multi-line top banner preventing any text truncation
    let header_lines = if area.width < 75 {
        vec![
            Line::from(vec![
                Span::styled("⚡ SPEEDTEST-RS", Style::default().fg(THEME.cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" │ IP: ", Style::default().fg(THEME.text_muted)),
                Span::styled(ip_str, Style::default().fg(THEME.text_main).add_modifier(Modifier::BOLD)),
                Span::styled(" │ Edge: ", Style::default().fg(THEME.text_muted)),
                Span::styled(colo_str, Style::default().fg(THEME.violet).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("ISP: ", Style::default().fg(THEME.text_muted)),
                Span::styled(org_str, Style::default().fg(THEME.amber)),
                Span::styled(" │ Loc: ", Style::default().fg(THEME.text_muted)),
                Span::styled(loc_str, Style::default().fg(THEME.emerald)),
            ]),
        ]
    } else {
        vec![Line::from(vec![
            Span::styled("⚡ SPEEDTEST-RS ", Style::default().fg(THEME.cyan).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(THEME.card_border)),
            Span::styled("IP: ", Style::default().fg(THEME.text_muted)),
            Span::styled(ip_str, Style::default().fg(THEME.text_main).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(THEME.card_border)),
            Span::styled("ISP: ", Style::default().fg(THEME.text_muted)),
            Span::styled(org_str, Style::default().fg(THEME.amber)),
            Span::styled(" │ ", Style::default().fg(THEME.card_border)),
            Span::styled("Loc: ", Style::default().fg(THEME.text_muted)),
            Span::styled(loc_str, Style::default().fg(THEME.emerald)),
            Span::styled(" │ ", Style::default().fg(THEME.card_border)),
            Span::styled("Edge: ", Style::default().fg(THEME.text_muted)),
            Span::styled(colo_str, Style::default().fg(THEME.violet).add_modifier(Modifier::BOLD)),
        ])]
    };

    let header = Paragraph::new(header_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(THEME.blue)),
        )
        .alignment(if area.width < 75 { Alignment::Center } else { Alignment::Left })
        .wrap(Wrap { trim: true });

    f.render_widget(header, area);
}

fn render_latency_row(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Ping Latency
            Constraint::Percentage(25), // Jitter
            Constraint::Percentage(25), // Packet Loss
            Constraint::Percentage(25), // Overall Progress
        ])
        .split(area);

    let stats = &app.telemetry.ping_stats;

    // Ping
    let ping_val = if stats.avg_ms > 0.0 {
        format!("{:.1} ms", stats.avg_ms)
    } else {
        "--".to_string()
    };
    let ping_sub = format!("Min: {:.1}ms │ Max: {:.1}ms", stats.min_ms, stats.max_ms);
    let ping_widget = Paragraph::new(vec![
        Line::from(Span::styled(ping_val, Style::default().fg(THEME.emerald).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(ping_sub, Style::default().fg(THEME.text_muted))),
    ])
    .block(
        Block::default()
            .title(" ⏱  Ping ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(THEME.emerald)),
    )
    .alignment(Alignment::Center);
    f.render_widget(ping_widget, chunks[0]);

    // Jitter
    let jitter_val = if stats.jitter_ms > 0.0 {
        format!("{:.2} ms", stats.jitter_ms)
    } else {
        "--".to_string()
    };
    let jitter_rating = if stats.jitter_ms <= 0.0 {
        "Waiting".to_string()
    } else if stats.jitter_ms < 5.0 {
        "Excellent (<5ms)".to_string()
    } else if stats.jitter_ms < 15.0 {
        "Good (<15ms)".to_string()
    } else {
        "High Jitter".to_string()
    };
    let jitter_widget = Paragraph::new(vec![
        Line::from(Span::styled(jitter_val, Style::default().fg(THEME.amber).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(jitter_rating, Style::default().fg(THEME.text_muted))),
    ])
    .block(
        Block::default()
            .title(" 〰  Jitter ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(THEME.amber)),
    )
    .alignment(Alignment::Center);
    f.render_widget(jitter_widget, chunks[1]);

    // Packet Loss
    let loss_val = format!("{:.1}%", stats.packet_loss_pct);
    let loss_color = if stats.packet_loss_pct > 1.0 {
        THEME.rose
    } else {
        THEME.cyan
    };
    let loss_samples = format!("Probes: {} OK", stats.samples.len());
    let loss_widget = Paragraph::new(vec![
        Line::from(Span::styled(loss_val, Style::default().fg(loss_color).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(loss_samples, Style::default().fg(THEME.text_muted))),
    ])
    .block(
        Block::default()
            .title(" 📦 Loss ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(loss_color)),
    )
    .alignment(Alignment::Center);
    f.render_widget(loss_widget, chunks[2]);

    // Stage Progress
    let (stage_title, stage_pct) = match app.telemetry.status {
        AppState::Idle => ("Idle", 0),
        AppState::Finished => ("Done", 100),
        AppState::Running => {
            if app.telemetry.ping_progress < 1.0 {
                ("Ping", (app.telemetry.ping_progress * 100.0) as u16)
            } else if app.telemetry.download_progress < 1.0 {
                ("Down", (app.telemetry.download_progress * 100.0) as u16)
            } else {
                ("Up", (app.telemetry.upload_progress * 100.0) as u16)
            }
        }
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(" 🔄 Progress ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(THEME.violet)),
        )
        .gauge_style(Style::default().fg(THEME.violet).bg(THEME.card_border))
        .percent(stage_pct)
        .label(format!("{} ({}%)", stage_title, stage_pct));
    f.render_widget(gauge, chunks[3]);
}

fn render_bandwidth_row(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let unit = app.telemetry.unit;

    // --- DOWNLOAD BOX ---
    let dl_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(21), Constraint::Min(8)])
        .split(chunks[0]);

    let dl_curr = unit.format_rate(app.telemetry.download_current_mbps);
    let dl_avg = unit.format_rate(app.telemetry.download_avg_mbps);
    let dl_peak = unit.format_rate(app.telemetry.download_peak_mbps);
    let dl_trans = format!("{:.1} MB", app.telemetry.download_bytes as f64 / 1_000_000.0);

    let dl_text = vec![
        Line::from(vec![
            Span::styled("Cur:  ", Style::default().fg(THEME.text_muted)),
            Span::styled(dl_curr, Style::default().fg(THEME.cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Avg:  ", Style::default().fg(THEME.text_muted)),
            Span::styled(dl_avg, Style::default().fg(THEME.text_main)),
        ]),
        Line::from(vec![
            Span::styled("Peak: ", Style::default().fg(THEME.text_muted)),
            Span::styled(dl_peak, Style::default().fg(THEME.emerald)),
        ]),
        Line::from(vec![
            Span::styled("Data: ", Style::default().fg(THEME.text_muted)),
            Span::styled(dl_trans, Style::default().fg(THEME.amber)),
        ]),
    ];

    let dl_block = Block::default()
        .title(" 📥  Download ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.cyan));

    f.render_widget(dl_block, chunks[0]);

    let dl_metrics_para = Paragraph::new(dl_text);
    f.render_widget(dl_metrics_para, Rect {
        x: dl_chunks[0].x + 1,
        y: dl_chunks[0].y + 1,
        width: dl_chunks[0].width.saturating_sub(1),
        height: dl_chunks[0].height.saturating_sub(2),
    });

    let dl_chart_rect = Rect {
        x: dl_chunks[1].x,
        y: dl_chunks[1].y + 1,
        width: dl_chunks[1].width.saturating_sub(1),
        height: dl_chunks[1].height.saturating_sub(2),
    };

    if app.use_braille && dl_chart_rect.width > 2 && dl_chart_rect.height > 1 {
        let max_val = app
            .telemetry
            .download_points
            .iter()
            .copied()
            .fold(10.0f64, f64::max);

        let points = &app.telemetry.download_points;
        let count = points.len();

        let canvas = Canvas::default()
            .background_color(ratatui::style::Color::Reset)
            .x_bounds([0.0, (count.max(20)) as f64])
            .y_bounds([0.0, max_val * 1.1])
            .paint(move |ctx| {
                if count > 1 {
                    for i in 0..count - 1 {
                        ctx.draw(&CanvasLine {
                            x1: i as f64,
                            y1: points[i],
                            x2: (i + 1) as f64,
                            y2: points[i + 1],
                            color: THEME.cyan,
                        });
                    }
                } else if count == 1 {
                    ctx.print(0.0, points[0], "•");
                }
            });
        f.render_widget(canvas, dl_chart_rect);
    } else {
        // Fallback: Standard blocky Sparkline
        let dl_sparkline = Sparkline::default()
            .data(&app.telemetry.download_history)
            .style(Style::default().fg(THEME.cyan))
            .max(
                app.telemetry
                    .download_history
                    .iter()
                    .copied()
                    .max()
                    .unwrap_or(10)
                    .max(10),
            );
        f.render_widget(dl_sparkline, dl_chart_rect);
    }

    // --- UPLOAD BOX ---
    let ul_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(21), Constraint::Min(8)])
        .split(chunks[1]);

    let ul_curr = unit.format_rate(app.telemetry.upload_current_mbps);
    let ul_avg = unit.format_rate(app.telemetry.upload_avg_mbps);
    let ul_peak = unit.format_rate(app.telemetry.upload_peak_mbps);
    let ul_trans = format!("{:.1} MB", app.telemetry.upload_bytes as f64 / 1_000_000.0);

    let ul_text = vec![
        Line::from(vec![
            Span::styled("Cur:  ", Style::default().fg(THEME.text_muted)),
            Span::styled(ul_curr, Style::default().fg(THEME.peach).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Avg:  ", Style::default().fg(THEME.text_muted)),
            Span::styled(ul_avg, Style::default().fg(THEME.text_main)),
        ]),
        Line::from(vec![
            Span::styled("Peak: ", Style::default().fg(THEME.text_muted)),
            Span::styled(ul_peak, Style::default().fg(THEME.violet)),
        ]),
        Line::from(vec![
            Span::styled("Data: ", Style::default().fg(THEME.text_muted)),
            Span::styled(ul_trans, Style::default().fg(THEME.amber)),
        ]),
    ];

    let ul_block = Block::default()
        .title(" 📤  Upload ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.peach));

    f.render_widget(ul_block, chunks[1]);

    let ul_metrics_para = Paragraph::new(ul_text);
    f.render_widget(ul_metrics_para, Rect {
        x: ul_chunks[0].x + 1,
        y: ul_chunks[0].y + 1,
        width: ul_chunks[0].width.saturating_sub(1),
        height: ul_chunks[0].height.saturating_sub(2),
    });

    let ul_chart_rect = Rect {
        x: ul_chunks[1].x,
        y: ul_chunks[1].y + 1,
        width: ul_chunks[1].width.saturating_sub(1),
        height: ul_chunks[1].height.saturating_sub(2),
    };

    if app.use_braille && ul_chart_rect.width > 2 && ul_chart_rect.height > 1 {
        let max_val = app
            .telemetry
            .upload_points
            .iter()
            .copied()
            .fold(10.0f64, f64::max);

        let points = &app.telemetry.upload_points;
        let count = points.len();

        let canvas = Canvas::default()
            .background_color(ratatui::style::Color::Reset)
            .x_bounds([0.0, (count.max(20)) as f64])
            .y_bounds([0.0, max_val * 1.1])
            .paint(move |ctx| {
                if count > 1 {
                    for i in 0..count - 1 {
                        ctx.draw(&CanvasLine {
                            x1: i as f64,
                            y1: points[i],
                            x2: (i + 1) as f64,
                            y2: points[i + 1],
                            color: THEME.peach,
                        });
                    }
                } else if count == 1 {
                    ctx.print(0.0, points[0], "•");
                }
            });
        f.render_widget(canvas, ul_chart_rect);
    } else {
        // Fallback: Standard blocky Sparkline
        let ul_sparkline = Sparkline::default()
            .data(&app.telemetry.upload_history)
            .style(Style::default().fg(THEME.peach))
            .max(
                app.telemetry
                    .upload_history
                    .iter()
                    .copied()
                    .max()
                    .unwrap_or(10)
                    .max(10),
            );
        f.render_widget(ul_sparkline, ul_chart_rect);
    }
}

fn render_details_and_logs(f: &mut Frame, app: &App, area: Rect) {
    let chunks = if area.width < 90 {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area)
    };

    // Left: Detailed Metrics Table
    let unit = app.telemetry.unit;
    let unit_str = match unit {
        SpeedUnit::Mbps => "Mbps",
        SpeedUnit::MBs => "MB/s",
    };

    let dl_avg_str = unit.format_rate(app.telemetry.download_avg_mbps);
    let dl_peak_str = unit.format_rate(app.telemetry.download_peak_mbps);
    let ul_avg_str = unit.format_rate(app.telemetry.upload_avg_mbps);
    let ul_peak_str = unit.format_rate(app.telemetry.upload_peak_mbps);
    let ping_avg_str = format!("{:.2} ms", app.telemetry.ping_stats.avg_ms);
    let ping_range_str = format!("Min: {:.1}ms, Max: {:.1}ms", app.telemetry.ping_stats.min_ms, app.telemetry.ping_stats.max_ms);
    let jitter_str = format!("{:.2} ms", app.telemetry.ping_stats.jitter_ms);
    let loss_str = format!("{:.1}%", app.telemetry.ping_stats.packet_loss_pct);
    let probes_str = format!("Total probes: {}", app.telemetry.ping_stats.samples.len());
    let total_data_str = format!("{:.2} MB", (app.telemetry.download_bytes + app.telemetry.upload_bytes) as f64 / 1_000_000.0);
    let data_breakdown_str = format!("DL: {:.1} MB / UL: {:.1} MB", app.telemetry.download_bytes as f64 / 1_000_000.0, app.telemetry.upload_bytes as f64 / 1_000_000.0);

    let table_rows = vec![
        Row::new(vec![
            "Download Speed (Avg)",
            dl_avg_str.as_str(),
            "Cloudflare CDN edge",
        ]).style(Style::default().fg(THEME.text_main)),
        Row::new(vec![
            "Download Peak",
            dl_peak_str.as_str(),
            "Max interval burst",
        ]).style(Style::default().fg(THEME.text_main)),
        Row::new(vec![
            "Upload Speed (Avg)",
            ul_avg_str.as_str(),
            "Multi-stream HTTP POST",
        ]).style(Style::default().fg(THEME.text_main)),
        Row::new(vec![
            "Upload Peak",
            ul_peak_str.as_str(),
            "Max upload burst",
        ]).style(Style::default().fg(THEME.text_main)),
        Row::new(vec![
            "Latency / Ping (Avg)",
            ping_avg_str.as_str(),
            ping_range_str.as_str(),
        ]).style(Style::default().fg(THEME.text_main)),
        Row::new(vec![
            "Jitter (Delay Var.)",
            jitter_str.as_str(),
            "RFC 3550 statistical",
        ]).style(Style::default().fg(THEME.text_main)),
        Row::new(vec![
            "Packet Loss Rate",
            loss_str.as_str(),
            probes_str.as_str(),
        ]).style(Style::default().fg(THEME.text_main)),
        Row::new(vec![
            "Data Transferred",
            total_data_str.as_str(),
            data_breakdown_str.as_str(),
        ]).style(Style::default().fg(THEME.text_main)),
    ];

    let header_row = Row::new(vec!["Metric", "Value", "Diagnostics"])
        .style(Style::default().fg(THEME.amber).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let table = Table::new(
        table_rows,
        [
            Constraint::Length(21),
            Constraint::Length(16),
            Constraint::Min(14),
        ],
    )
    .header(header_row)
    .block(
        Block::default()
            .title(format!(" 📊 Telemetry [{}] ", unit_str))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(THEME.card_border)),
    );

    f.render_widget(table, chunks[0]);

    // Right: Live Event / Diagnostic Logs
    let log_lines: Vec<Line> = app
        .telemetry
        .logs
        .iter()
        .rev()
        .take(15)
        .map(|l| Line::from(Span::styled(l, Style::default().fg(THEME.text_muted))))
        .collect();

    let logs_widget = Paragraph::new(log_lines)
        .block(
            Block::default()
                .title(" 📜 Live Diagnostics Log ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(THEME.card_border)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(logs_widget, chunks[1]);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let status_color = match app.telemetry.status {
        AppState::Idle => THEME.text_muted,
        AppState::Running => THEME.amber,
        AppState::Finished => THEME.emerald,
    };

    let unit_tag = match app.telemetry.unit {
        SpeedUnit::Mbps => "Mbps",
        SpeedUnit::MBs => "MB/s",
    };

    let notif = app
        .telemetry
        .export_notification
        .as_deref()
        .unwrap_or(&app.telemetry.status_message);

    let chart_tag = if app.use_braille { "Braille" } else { "Block" };

    // Dynamic responsive footer:
    // If window width is compact (< 75 cols), use ultra-compact labels
    let controls_line = if area.width < 75 {
        Line::from(vec![
            Span::styled("[Space] ", Style::default().fg(THEME.cyan).add_modifier(Modifier::BOLD)),
            Span::styled("Run ", Style::default().fg(THEME.text_main)),
            Span::styled("[P] ", Style::default().fg(THEME.emerald).add_modifier(Modifier::BOLD)),
            Span::styled("Ping ", Style::default().fg(THEME.text_main)),
            Span::styled("[D] ", Style::default().fg(THEME.cyan).add_modifier(Modifier::BOLD)),
            Span::styled("Down ", Style::default().fg(THEME.text_main)),
            Span::styled("[U] ", Style::default().fg(THEME.peach).add_modifier(Modifier::BOLD)),
            Span::styled("Up ", Style::default().fg(THEME.text_main)),
            Span::styled("[T] ", Style::default().fg(THEME.amber).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} ", unit_tag), Style::default().fg(THEME.text_main)),
            Span::styled("[B] ", Style::default().fg(THEME.cyan).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} ", chart_tag), Style::default().fg(THEME.text_main)),
            Span::styled("[E] ", Style::default().fg(THEME.emerald).add_modifier(Modifier::BOLD)),
            Span::styled("Save ", Style::default().fg(THEME.text_main)),
            Span::styled("[Q] ", Style::default().fg(THEME.violet).add_modifier(Modifier::BOLD)),
            Span::styled("Quit", Style::default().fg(THEME.text_main)),
        ])
    } else {
        Line::from(vec![
            Span::styled("[Space] ", Style::default().fg(THEME.cyan).add_modifier(Modifier::BOLD)),
            Span::styled("Run All   ", Style::default().fg(THEME.text_main)),
            Span::styled("[P] ", Style::default().fg(THEME.emerald).add_modifier(Modifier::BOLD)),
            Span::styled("Ping   ", Style::default().fg(THEME.text_main)),
            Span::styled("[D] ", Style::default().fg(THEME.cyan).add_modifier(Modifier::BOLD)),
            Span::styled("Download   ", Style::default().fg(THEME.text_main)),
            Span::styled("[U] ", Style::default().fg(THEME.peach).add_modifier(Modifier::BOLD)),
            Span::styled("Upload   ", Style::default().fg(THEME.text_main)),
            Span::styled("[T] ", Style::default().fg(THEME.amber).add_modifier(Modifier::BOLD)),
            Span::styled(format!("Unit: {}   ", unit_tag), Style::default().fg(THEME.text_main)),
            Span::styled("[B] ", Style::default().fg(THEME.cyan).add_modifier(Modifier::BOLD)),
            Span::styled(format!("Chart: {}   ", chart_tag), Style::default().fg(THEME.text_main)),
            Span::styled("[E] ", Style::default().fg(THEME.emerald).add_modifier(Modifier::BOLD)),
            Span::styled("Export JSON   ", Style::default().fg(THEME.text_main)),
            Span::styled("[C] ", Style::default().fg(THEME.rose).add_modifier(Modifier::BOLD)),
            Span::styled("Cancel   ", Style::default().fg(THEME.text_main)),
            Span::styled("[Q] ", Style::default().fg(THEME.violet).add_modifier(Modifier::BOLD)),
            Span::styled("Quit", Style::default().fg(THEME.text_main)),
        ])
    };

    let status_line = Line::from(vec![
        Span::styled("● ", Style::default().fg(status_color)),
        Span::styled(notif, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
    ]);

    let footer = Paragraph::new(vec![controls_line, status_line])
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(footer, area);
}
