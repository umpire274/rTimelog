# rTimelog

[![Crates.io](https://img.shields.io/crates/v/rTimelog.svg)](https://crates.io/crates/rTimelog)
[![Build Status](https://github.com/umpire274/rTimelog/actions/workflows/ci.yml/badge.svg)](https://github.com/umpire274/rTimelog/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/umpire274/rTimelog/branch/main/graph/badge.svg?token=5fb9648b-1dd5-46d2-8130-3916505ef4a3)](https://codecov.io/gh/umpire274/rTimelog)

`rTimelog` is a simple and cross-platform **command-line tool** written in Rust that helps you track your working hours, lunch breaks, and calculate expected exit times and surplus work time.

## ✨ Features

- Add daily work sessions with:
  - Date
  - Start time
  - Lunch break duration
  - End time
- Automatically calculate:
  - **Expected exit time** (based on 8h work + lunch break, max 1h)
  - **Surplus** (actual exit – expected exit)
- Store data in a lightweight **SQLite database** (embedded, no server required)
- Fully cross-platform: Windows, Linux, macOS

## 📦 Installation

### From source
Clone this repository and build with Cargo:

```bash
git clone https://github.com/umpire274/rTimelog.git
cd rTimelog
cargo build --release
```

The compiled binary will be available in:

- `target/release/rTimelog` (Linux/macOS)
- `target\release\rTimelog.exe` (Windows)

### Future plans
- [ ] Publish on [crates.io](https://crates.io)  
- [ ] Prebuilt binaries for GitHub Releases

## 🚀 Usage

Initialize the database:

```bash
rTimelog init
```

Add a work session:

```bash
rTimelog add 2025-09-12 09:00 45 17:30
# or simply (defaults to today’s date)
rTimelog add 09:00 45 17:30
```

List all sessions:

```bash
rTimelog list
```

Example output:

```
📅 Saved sessions:
  1: 2025-09-12 | Start 09:00 | Lunch 45 min | End 17:30 | Expected 18:45 | Surplus -75 min
  2: 2025-09-13 | Start 08:30 | Lunch 60 min | End 17:45 | Expected 17:30 | Surplus +15 min
```

## 🛠 Dependencies

- [rusqlite](https://crates.io/crates/rusqlite) (with `bundled` feature for cross-platform support)
- [chrono](https://crates.io/crates/chrono) (date & time handling)
- [clap](https://crates.io/crates/clap) (command-line argument parsing)

## 📋 Roadmap

- [ ] Export data to CSV/JSON
- [ ] Daily/weekly/monthly reports
- [ ] Configurable workday duration
- [ ] Integration with GUI or TUI frontend

## 📜 License

Licensed under the MIT License.  
See [LICENSE](LICENSE) for details.
