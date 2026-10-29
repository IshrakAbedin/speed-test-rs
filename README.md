# ⚡ speed-test-rs

A modern, comprehensive, and beautiful Terminal User Interface (TUI) application built in Rust using [`ratatui`](https://crates.io/crates/ratatui) and [`crossterm`](https://crates.io/crates/crossterm) for measuring network performance metrics.

> This app is built with Google Antigravity CLI using Gemini Flash 3.8

---

## ✨ Features

- **⏱ Latency & Ping**: Real-time Round Trip Time (Min, Avg, Max).
- **〰 Jitter (RFC 3550)**: Delay variation calculated with statistical exponential smoothing.
- **📦 Packet Loss**: Probe failure percentage and sample counter.
- **📈 Granular Braille Charts (btop-style)**: High-resolution $2\times 4$ subpixel Braille dot graphs (`Canvas`) with automatic fallback to block sparklines on terminals lacking UTF-8 support. Toggle live with `B` or pass `--no-braille`.
- **📥 Download Benchmark**: Concurrent multi-stream HTTP benchmarking with real-time transfer sparklines.
- **📤 Upload Benchmark**: Concurrent zero-overhead HTTP payload uploads.
- **🌐 Network & ISP Discovery**: Resolves Public IP, ISP/AS Organization, City, Region, Country, and nearest CDN Edge PoP code.
- **📊 Detailed Telemetry Table**: Clean breakdown of all measured network metrics.
- **📜 Live Diagnostics Activity Log**: Scrolling log of engine probe events and timestamps.
- **💾 JSON Export**: Press `E` to export comprehensive test results to JSON.
- **🎛 Flexible Unit Toggling**: Switch between `Mbps` (Megabits/sec) and `MB/s` (Megabytes/sec) with `T`.
- **⌨ Interactive Controls**:
  - `Space` / `Enter` / `R`: Run full benchmark suite.
  - `P`: Measure Latency & Jitter only.
  - `D`: Benchmark Download throughput only.
  - `U`: Benchmark Upload throughput only.
  - `T`: Toggle speed units (Mbps / MB/s).
  - `E`: Export full benchmark report to JSON.
  - `C`: Cancel ongoing test.
  - `Q` / `Ctrl+C`: Quit application cleanly.

---

## 🚀 Quick Start

### Build & Run

Ensure Rust and Cargo are installed on your system.

```bash
# Build the standard release binary
cargo build --release

# Build fully standalone, statically-linked Musl binary
cargo build --release --target x86_64-unknown-linux-musl
```

The resulting Musl binary at `target/x86_64-unknown-linux-musl/release/speed-test-rs` is completely statically linked (`statically linked`, zero libc/glibc dependencies) and runs portably across any Linux distribution.

Or run directly via Cargo:

```bash
cargo run --release
```

### Command Line Options

```text
Usage: speed-test-rs [OPTIONS]

Options:
  -d, --duration <DURATION>        Test duration per phase in seconds [default: 10]
  -c, --concurrency <CONCURRENCY>  Number of concurrent streams [default: 6]
  -a, --auto-start                 Automatically start full test on launch [default: true]
  -h, --help                       Print help
  -V, --version                    Print version
```

Example customizing duration and stream concurrency:
```bash
speed-test-rs -d 15 -c 8
```

---

## 📐 Architecture

- `src/engine/`: Core asynchronous networking module:
  - `info.rs`: IP, ISP, geographic location, and CDN PoP edge detection.
  - `latency.rs`: Latency probes, packet loss, and RFC 3550 jitter calculation.
  - `download.rs`: Concurrent chunk stream download benchmark.
  - `upload.rs`: Concurrent payload upload benchmark.
  - `mod.rs`: Test pipeline coordinator.
- `src/state.rs`: Telemetry state, metrics history, event queue, and JSON exporter.
- `src/ui/`: Ratatui view layer rendering sparklines, gauges, tables, and headers.
- `src/app.rs`: Application controller handling user keystrokes and async event dispatching.
- `src/main.rs`: Terminal initialization, event loop, and teardown.

---

## 📄 License
MIT
