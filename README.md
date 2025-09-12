# rTimelog

[![Build Status](https://github.com/umpire274/rTimelog/actions/workflows/ci.yml/badge.svg)](https://github.com/umpire274/rTimelog/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/umpire274/rTimelog)](https://github.com/umpire274/rTimelog/releases)
[![codecov](https://codecov.io/gh/umpire274/rTimelog/branch/main/graph/badge.svg?token=5fb9648b-1dd5-46d2-8130-3916505ef4a3)](https://codecov.io/gh/umpire274/rTimelog)

`rTimelog` is a simple, cross-platform **command-line tool** written in Rust that helps you track your working hours, lunch breaks, expected exit times, and surplus work time.

---

## ✨ Features

- Add daily work sessions with:
  - Date (`YYYY-MM-DD`)
  - Start time (`HH:MM`)
  - Lunch break duration (min. **30 min**, max. **90 min**)
  - End time (`HH:MM`)
- Automatically calculate:
  - **Expected exit time** (8h work + lunch break, with lunch normalization rules)
  - **Surplus** (actual exit – expected exit, shows `+` for positive values)
- Query sessions:
  - All sessions
  - By year (`--period 2025`)
  - By year and month (`--period 2025-09`)
- Store data in a lightweight **SQLite database** (embedded, no server required)
- Fully cross-platform: Windows, Linux, macOS

---

## 📦 Installation

### Prebuilt binaries
Download the latest release from the [GitHub Releases](https://github.com/umpire274/rTimelog/releases).  
Available for:
- **Linux** (`.tar.gz`)
- **macOS Intel** (`.tar.gz`)
- **macOS Apple Silicon** (`.tar.gz`)
- **Windows** (`.zip`)

Each package includes:
- The compiled binary
- `README.md`
- `LICENSE`
- `CHANGELOG.md`

### From source
Clone this repository and build with Cargo:

```bash
git clone https://github.com/umpire274/rTimelog.git
cd rTimelog
cargo build --release
```

The compiled binary will be available in:

- `target/release/rtimelog` (Linux/macOS)
- `target\release\rtimelog.exe` (Windows)

---

## 🚀 Usage

Initialize the database:

```bash
rtimelog init
```

Add a work session:

```bash
rtimelog add 2025-09-12 09:00 45 17:30
```

List all sessions:

```bash
rtimelog list
```

Filter by year:

```bash
rtimelog list --period 2025
```

Filter by year + month:

```bash
rtimelog list --period 2025-09
```

Example output:

```
📅 Saved sessions for September 2025:
  1: 2025-09-12 | Start 09:00 | Lunch 45 min | End 17:30 | Expected 18:45 | Surplus -75 min
  2: 2025-09-13 | Start 08:30 | Lunch 60 min | End 17:45 | Expected 17:30 | Surplus +15 min
```

---

## 🛠 Dependencies

- [rusqlite](https://crates.io/crates/rusqlite) (with `bundled` feature for cross-platform SQLite)
- [chrono](https://crates.io/crates/chrono) (date & time handling)
- [clap](https://crates.io/crates/clap) (command-line argument parsing)

---

## 📋 Roadmap

- [ ] Export data to CSV/JSON
- [ ] Daily/weekly/monthly reports
- [ ] Configurable workday duration
- [ ] Optional localization (month names, language for output)
- [ ] Integration with GUI or TUI frontend

---

## 📜 License

Licensed under the MIT License.  
See [LICENSE](LICENSE) for details.
