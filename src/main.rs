mod app;
mod engine;
mod state;
mod ui;

use std::io;
use std::time::Duration;
use clap::Parser;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::App;
use crate::engine::TestPhaseToRun;

#[derive(Parser, Debug)]
#[command(author, version, about = "A beautiful TUI for measuring Internet speed, jitter, and latency", long_about = None)]
struct Args {
    /// Test duration per phase (download/upload) in seconds
    #[arg(short, long, default_value_t = 10)]
    duration: u64,

    /// Number of concurrent connection streams
    #[arg(short, long, default_value_t = 6)]
    concurrency: usize,

    /// Automatically start full speed test on launch
    #[arg(short, long, default_value_t = true)]
    auto_start: bool,

    /// Force standard blocky charts instead of high-res Braille font
    #[arg(long, default_value_t = false)]
    no_braille: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(args.duration, args.concurrency, args.no_braille);

    if args.auto_start {
        app.start_test(TestPhaseToRun::All);
    }

    // Main event loop
    let tick_rate = Duration::from_millis(50);
    let mut last_tick = std::time::Instant::now();

    loop {
        // Drain events from background runner
        app.process_events();

        terminal.draw(|f| ui::draw(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key);
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = std::time::Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    // Teardown terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    println!("speedtest-rs closed cleanly. Thank you!");
    Ok(())
}
