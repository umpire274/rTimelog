# rTimelog

[![Build Status](https://github.com/umpire274/rTimelog/actions/workflows/ci.yml/badge.svg)](https://github.com/umpire274/rTimelog/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/umpire274/rTimelog)](https://github.com/umpire274/rTimelog/releases)
[![codecov](https://codecov.io/gh/umpire274/rTimelog/branch/main/graph/badge.svg?token=5fb9648b-1dd5-46d2-8130-3916505ef4a3)](https://codecov.io/gh/umpire274/rTimelog)

`rTimelog` is a simple, cross-platform **command-line tool** written in Rust to track daily working sessions, including working position, start and end times, and lunch breaks.  
The tool calculates the expected exit time and the surplus of worked minutes.

---

## ✨ Features

- Store and manage working sessions in a SQLite database
- Track working position:
  - `O` = Office
  - `R` = Remote
- Add or update sessions with flags:
  - `--pos` working position
  - `--in` start time (HH:MM)
  - `--lunch` lunch duration in minutes (0–90)
  - `--out` end time (HH:MM)
- Automatic normalization of lunchtime if working from the Office:
  - Minimum 30 minutes
  - Maximum 90 minutes
  - If less than 30 minutes are taken, the missing time is added to the expected exit time
- Global option `--db` to choose the SQLite database file:
  - If a **filename** is provided, it will be placed under the rTimelog config directory:
    - Linux/macOS: `$HOME/.rtimelog/<filename>`
    - Windows: `%APPDATA%\rtimelog\<filename>`
  - If an **absolute path** is provided, that file will be used directly
- If no sessions are stored, `rtimelog list` prints:  
  ```
  ⚠️ No recorded sessions found
  ```

---

## 🚀 Usage

### Initialize the database

```bash
# Default path (~/.rtimelog/rtimelog.sqlite or %APPDATA%\rtimelog\rtimelog.sqlite)
rtimelog init

# Initialize with custom DB name inside config dir
rtimelog --db custom.sqlite init

# Initialize with absolute path
rtimelog --db /tmp/test.sqlite init

# Initialize with an absolute path containing spaces (Windows version)
rtimelog --db "G:\My Drive\Work\ACMEinc\timetable\rtimelog.sqlite" init
```

### Add sessions

```bash
# Add a complete session
rtimelog add 2025-09-12 --pos O --in 09:00 --lunch 45 --out 17:30

# Add only the start time
rtimelog add 2025-09-12 --pos O --in 09:00

# Add the lunch break later
rtimelog add 2025-09-12 --lunch 45

# Add the exit time
rtimelog add 2025-09-12 --out 17:30
```

### List sessions

```bash
# List all sessions
rtimelog list

# List sessions for a specific year
rtimelog list --period 2025

# List sessions for a specific year and month
rtimelog list --period 2025-09
```

---

## 📊 Example output

```
📅 Saved sessions for September 2025:
  1: 2025-09-12 | Pos O | Start 09:00 | Lunch 45 min | End 17:30 | Expected 18:45 | Surplus -75 min
  2: 2025-09-13 | Pos R | Start 08:30 | Lunch 60 min | End 17:45 | Expected 17:30 | Surplus +15 min
```

---

## 🛠️ Build

Clone the repository and build with Cargo:

```bash
git clone https://github.com/umpire274/rTimelog.git
cd rTimelog
cargo build --release
```

The binary will be in `target/release/rtimelog`.

---

## Development / Testing

For integration tests or development purposes, you can use the `--test` flag.
This ensures that no configuration file (`rtimelog.conf`) is written or modified
in your user directory.

Example:

```bash
cargo run -- --db /tmp/test.sqlite --test init
cargo run -- --db /tmp/test.sqlite --test add 2025-09-15 O 09:00 30 17:00
cargo run -- --db /tmp/test.sqlite --test list
```

---

## 📄 License

This project is licensed under the MIT License.  
See [LICENSE](LICENSE) for details.
