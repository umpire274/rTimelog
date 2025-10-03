# rTimelog

[![Build Status](https://github.com/umpire274/rTimelog/actions/workflows/ci.yml/badge.svg)](https://github.com/umpire274/rTimelog/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/umpire274/rTimelog)](https://github.com/umpire274/rTimelog/releases)
[![codecov](https://codecov.io/gh/umpire274/rTimelog/graph/badge.svg?token=5WPQF58D5Z)](https://codecov.io/gh/umpire274/rTimelog)

`rTimelog` is a simple, cross-platform **command-line tool** written in Rust to track daily working sessions, including
working position, start and end times, and lunch breaks.  
The tool calculates the expected exit time and the surplus of worked minutes.

---

## What's new in v0.4.1

**Refactor, fixes and display improvements**

This micro-release cleans up duplicated code, improves testability and the user-facing duration format:

- Refactored: extracted the `create_missing_event` helper into `src/events.rs` to make it reusable and easier to
  unit-test.
- Fixed: removed duplicated logic from `commands.rs` that caused compile-time errors, and consolidated event creation
  paths.
- Tests: added a focused unit test covering creation of missing events (now easier to test because the helper is exposed
  to the events module).
- Display: in `rtimelog list --events --summary` the `Dur` column is now shown as human-friendly hours/minutes (for
  example `2h 30m`) instead of raw minutes. This is a presentation-only change; JSON output and internal representations
  still use integer minutes (`duration_minutes`).
- Cleanup: removed temporary tracking of `commit_msg.txt` and ensured commit messages are kept in English.

```bash
# Detailed events (raw punches)
rtimelog list --events

# Summarized per pair (start/end/lunch/duration, human-friendly Dur)
rtimelog list --events --summary

# JSON summary (durations still in minutes)
rtimelog list --events --summary --json
```

### Sample (summary mode)

```
📊 Event pairs summary:
Date        Pair  Pos  Start  End    Lunch  Dur
----------  ----  ---  -----  -----  -----  --------
2025-12-01  1     O    09:00  12:00     30  2h 30m
2025-12-01  2     O    13:00  17:00      0  4h 00m
```

*Note: JSON output still contains `duration_minutes` expressed as integer minutes.*

---

## What's new in v0.3.6 *(previous)*

- Added: new `log` subcommand with `--print` to display rows from the internal `log` table for debugging and audit
  purposes.

---

## ✨ Features

- Add, update, delete and list work sessions.
- Track **start time**, **lunch duration**, and **end time**.
- Calculate **expected exit time** and **surplus** automatically.
- Manage multiple **working positions**:
    - `O` = **Office**
    - `R` = **Remote**
    - `C` = **On-Site (Client)**
    - `H` = **Holiday**
- Colorized output for better readability:
    - **Blue** = Office
    - **Cyan** = Remote
    - **Yellow** = On-Site (Client)
    - **Purple background + white bold** = Holiday
- Configurable default DB path via configuration file or `--db` parameter.
- Automatic DB migrations with version tracking (`schema_migrations` table).
- Configurable daily working time (`min_work_duration`, default `8h`).
- Automatic expected exit calculation based on:
    - Start time
    - Lunch break duration
    - Configured working time
- Automatic handling of lunch break rules:
    - Minimum 30 minutes
    - Maximum 1h 30m
    - Required only for `Office` position (`O`)
- View surplus/deficit of worked time compared to expected
- Display of the **total surplus** at the bottom of `list` output.
- **Event mode** with: Pair grouping, per-pair summary, JSON enrichment, unmatched detection, filtering by position &
  pair id.
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
It includes for current releases (≥ 0.4.0):

```yaml
database: /home/user/.rtimelog/rtimelog.sqlite
default_position: O
min_work_duration: 8h
min_duration_lunch_break: 30
max_duration_lunch_break: 90
separator_char: "-"
```

Key fields:

- **database** → path to the SQLite DB file
- **default_position** → default working position (`O`, `R`, `C`, `H`)
- **min_work_duration** → daily expected working time (e.g. `7h 36m`, `8h`)
- **min_duration_lunch_break** / **max_duration_lunch_break** → lunch constraints (minutes)
- **separator_char** → character used for month-end separator lines

> NOTE: Older docs referenced `working_time`; it has been unified as `min_work_duration`.

Override DB path at runtime:

```bash
rtimelog --db /custom/path/mydb.sqlite <command>
```

---

## 🖥️ Usage

### Initialize DB and config

```bash
rtimelog init
```

Custom DB file relative to config dir:

```bash
rtimelog --db mydb.sqlite init
```

Absolute path:

```bash
rtimelog --db "G:/My Drive/Work/Timelog/rtimelog.sqlite" init
```

### Add a full work session

```bash
rtimelog add 2025-09-13 O 09:00 60 17:30
```

Creates or updates the legacy session AND adds two events (in/out) for reporting.

### Partial updates (each creates/updates events when relevant)

```bash
rtimelog add 2025-09-13 --pos R
rtimelog add 2025-09-13 --in 09:00
rtimelog add 2025-09-13 --lunch 45
rtimelog add 2025-09-13 --out 17:30
```

### Add holiday

```bash
rtimelog add 2025-09-14 --pos H
```

### List sessions (legacy view)

```bash
rtimelog list                # all
rtimelog list --period 2025  # year
rtimelog list --period 2025-09  # year-month
rtimelog list --pos o        # position (case-insensitive)
```

### List raw events

```bash
rtimelog list --events
rtimelog list --events --pos r          # filter by position
rtimelog list --events --pairs 2        # only pair 2 (per date)
rtimelog list --events --json           # raw JSON with pair & unmatched
```

### Summarize events per pair

```bash
rtimelog list --events --summary
rtimelog list --events --summary --pairs 1
rtimelog list --events --summary --json
```

### Delete a session by id

```bash
rtimelog del 1
```

### Internal log

```bash
rtimelog log --print
```

---

## Event mode – behavior details

- **Pair numbering** restarts each date.
- **Unmatched** rows (only `in` or only `out`) show `*` and `duration_minutes = 0` in summary.
- **Lunch minutes** shown on the `out` event (and propagated to summary) if provided or auto-deduced.
- **Filtering precedence**: `--pairs` applies *after* computing pairs; combining with `--summary` reduces summary rows.
- **JSON schemas**:
    - Raw events: fields from DB + `pair`, `unmatched`.
    - Summary: `date, pair, position, start, end, lunch_minutes, duration_minutes, unmatched`.

---

## ⚙️ Configuration (duplicate quick ref)

(See above primary configuration section.)

---

## 🗄️ Database migrations

*(unchanged – see CHANGELOG for past versions)*

---

## ⚠️ Notes

- Lunch validation: min 30, max 90 (Office only mandatory). Remote can specify 0.
- Holidays ignore start/end/lunch; still appear in sessions listing.
- `--db` allows isolated datasets (useful for testing).

---

## 📊 Legacy session output example

```
📅 Saved sessions for September 2025:
  1: 2025-09-01 | Remote           | Start 09:08 | Lunch 00:30 | End 17:30 | Expected 17:14 | Surplus  +16 min
  2: 2025-09-04 | Office           | Start 09:35 | Lunch 00:30 | End 17:44 | Expected 17:41 | Surplus   +3 min
  3: 2025-09-05 | Remote           | Start 09:11 | Lunch 00:30 | End 17:01 | Expected 17:17 | Surplus  -16 min
  4: 2025-09-11 | Remote           | Start 08:08 | Lunch   -   | End 12:16 | Worked  4 h 08 min
  5: 2025-09-17 | Office           | Start 09:42 | Lunch 00:30 | End 17:28 | Expected 17:48 | Surplus  -20 min
  6: 2025-09-18 | Remote           | Start 10:50 | Lunch   -   | End   -   | Expected 18:56 | Surplus    -
  7: 2025-09-19 | Holiday          | Start   -   | Lunch   -   | End   -   | Expected   -   | Surplus    - min
  8: 2025-09-22 | Holiday          | Start   -   | Lunch   -   | End   -   | Expected   -   | Surplus    - min
```

---

## Output formatting: month-end separator

(See `separator_char` in configuration.)

---

## 🧪 Tests

Run all tests:

```bash
cargo test --all
```

Include coverage for: sessions CRUD, events pairing, summary, JSON, holidays, migrations.

---

## 📦 Installation

```bash
git clone https://github.com/umpire274/rTimelog.git
cd rTimelog
cargo build --release
```

Binaries in `target/release/` or use releases page.

---

## 📜 License

MIT License – see [LICENSE](LICENSE).

---

### Internal Log Recap

```bash
rtimelog log --print
```

Records concise audit lines for `init`, `add`, `del` and auto-lunch adjustments.
