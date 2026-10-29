use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::engine::{run_speed_test_pipeline, TestPhaseToRun};
use crate::state::{AppState, TelemetryState, TestEvent};

pub struct App {
    pub telemetry: TelemetryState,
    pub should_quit: bool,
    pub event_tx: UnboundedSender<TestEvent>,
    pub event_rx: UnboundedReceiver<TestEvent>,
    pub test_abort: Arc<AtomicBool>,
    pub phase_duration: Duration,
    pub concurrency: usize,
    pub use_braille: bool,
}

impl App {
    pub fn new(phase_duration_secs: u64, concurrency: usize, force_block: bool) -> Self {
        let (tx, rx) = unbounded_channel();
        // Detect UTF-8 / modern terminal support; fallback to block if ASCII/legacy or requested
        let supports_utf8 = std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .map(|l| l.to_lowercase().contains("utf-8") || l.to_lowercase().contains("utf8"))
            .unwrap_or(true);
        let term = std::env::var("TERM").unwrap_or_default();
        let is_dumb = term == "dumb" || term == "linux";

        let use_braille = !force_block && supports_utf8 && !is_dumb;

        Self {
            telemetry: TelemetryState::new(),
            should_quit: false,
            event_tx: tx,
            event_rx: rx,
            test_abort: Arc::new(AtomicBool::new(false)),
            phase_duration: Duration::from_secs(phase_duration_secs),
            concurrency,
            use_braille,
        }
    }

    pub fn start_test(&mut self, phase: TestPhaseToRun) {
        if self.telemetry.status == AppState::Running {
            return;
        }

        self.telemetry.status = AppState::Running;
        if phase == TestPhaseToRun::All {
            self.telemetry.reset_metrics();
        }

        self.test_abort.store(false, Ordering::Relaxed);
        let abort_clone = Arc::clone(&self.test_abort);
        let tx_clone = self.event_tx.clone();
        let duration = self.phase_duration;
        let concurrency = self.concurrency;

        tokio::spawn(async move {
            run_speed_test_pipeline(tx_clone, abort_clone, phase, duration, concurrency).await;
        });
    }

    pub fn cancel_test(&mut self) {
        if self.telemetry.status == AppState::Running {
            self.test_abort.store(true, Ordering::Relaxed);
            self.telemetry.status = AppState::Idle;
            self.telemetry.status_message = "Test cancelled by user.".into();
            self.telemetry.add_log("Test manually aborted.".into());
        }
    }

    pub fn process_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            self.telemetry.handle_event(event);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        // Ignore release and repeat events for single triggers
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
            }
            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('r') | KeyCode::Char('R') => {
                self.start_test(TestPhaseToRun::All);
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.start_test(TestPhaseToRun::LatencyOnly);
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                self.start_test(TestPhaseToRun::DownloadOnly);
            }
            KeyCode::Char('u') | KeyCode::Char('U') => {
                self.start_test(TestPhaseToRun::UploadOnly);
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                self.telemetry.unit = self.telemetry.unit.toggle();
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                self.use_braille = !self.use_braille;
                let mode = if self.use_braille {
                    "Braille (btop-style)"
                } else {
                    "Block (fallback)"
                };
                self.telemetry
                    .add_log(format!("Switched chart renderer to {}", mode));
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                self.cancel_test();
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                let filename = format!(
                    "speedtest_report_{}.json",
                    chrono::Local::now().format("%Y%m%d_%H%M%S")
                );
                if let Err(e) = self.telemetry.export_to_json(&filename) {
                    self.telemetry
                        .add_log(format!("Failed to export JSON: {}", e));
                }
            }
            _ => {}
        }
    }
}
