# rTimelog

[![Build Status](https://github.com/umpire274/rTimelog/actions/workflows/ci.yml/badge.svg)](https://github.com/umpire274/rTimelog/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/umpire274/rTimelog)](https://github.com/umpire274/rTimelog/releases)
[![codecov](https://codecov.io/gh/umpire274/rTimelog/graph/badge.svg?token=5WPQF58D5Z)](https://codecov.io/gh/umpire274/rTimelog)

`rTimelog` is a simple, cross-platform **command-line tool** written in Rust to track daily working sessions, including
working position, start and end times, and lunch breaks.  
The tool calculates the expected exit time and the surplus of worked minutes.

---

## ✨ Features

- Add work sessions with:
    - Start time
    - Lunch break duration
    - End time
    - Position (`O` = Office, `R` = Remote, `H` = Holiday)
- Configurable daily working time (default `8h`)
- Automatic expected exit calculation based on:
    - Start time
    - Lunch break duration
    - Configured working time
- Automatic handling of lunch break rules:
    - Minimum 30 minutes
    - Maximum 1h 30m
    - Required only for `Office` position (`O`)
- View surplus/deficit of worked time compared to expected
- Automatic database migration for schema changes
- Cross-platform configuration file management:
    - Linux/macOS: `$HOME/.rtimelog/rtimelog.conf`
    - Windows: `%APPDATA%\rtimelog\rtimelog.conf`

---

## ⚙️ Configuration

When you run:

```bash
rtimelog init
```

a configuration file is created in the user’s config directory (`rtimelog.conf`).  
It includes:

```yaml
database: /home/user/.rtimelog/rtimelog.sqlite
default_position: O
working_time: 8h
```

- **database** → path to the SQLite DB file
- **default_position** → default working position (`O` = Office, `R` = Remote, `H` = Holiday)
- **working_time** → daily expected working time, e.g.:
    - `8h`
    - `7h 36m`

You can override the DB path at runtime with the global option:

```bash
rtimelog --db /custom/path/mydb.sqlite <command>
```

---

## 🖥️ Usage

### Initialize DB and config

```bash
rtimelog init
```

With custom DB name:

```bash
rtimelog --db mydb.sqlite init
```

With absolute path (spaces allowed):

```bash
rtimelog --db "G:/My Drive/Work/Timelog/rtimelog.sqlite" init
```

---

### Add work session (full)

```bash
rtimelog add 2025-09-13 O 09:00 60 17:30
```

---

### Add work session (partial updates)

```bash
rtimelog add 2025-09-13 --pos R
rtimelog add 2025-09-13 --in 09:00
rtimelog add 2025-09-13 --lunch 45
rtimelog add 2025-09-13 --out 17:30
```

---

### Add holiday

```bash
rtimelog add 2025-09-14 --pos H
```

Output with purple background:

```
  5: 2025-09-14 | Holiday
```

---

### List sessions

All:

```bash
rtimelog list
```

By year:

```bash
rtimelog list --period 2025
```

By year and month:

```bash
rtimelog list --period 2025-09
```

---

### Show configuration file path

```bash
rtimelog conf --print
```

---

## 📊 Output example

```
📅 Saved sessions for September 2025:
  1: 2025-09-01 | Position O | Start 09:00 | Lunch 00:45 | End 17:30 | Expected 17:45 | Surplus -15 min
  2: 2025-09-02 | Position R | Start 08:45 | Lunch 00:00 | End 17:15 | Expected 16:45 | Surplus +30 min
  3: 2025-09-14 | Holiday
```

---

## 🧪 Tests

Run all tests:

```bash
cargo test --all
```

Tests include:

- DB initialization
- Adding and listing work sessions
- Handling of holidays
- Configurable working time
- Migration compatibility

---

## 📦 Installation

### From source

```bash
git clone https://github.com/umpire274/rTimelog.git
cd rTimelog
cargo build --release
```

Binaries will be in `target/release/`.

### Precompiled binaries

Available on the [GitHub Releases](https://github.com/umpire274/rTimelog/releases) page.

---

## 📜 License

MIT License.  
See [LICENSE](LICENSE) for details.
