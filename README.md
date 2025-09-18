# rTimelog

[![Build Status](https://github.com/umpire274/rTimelog/actions/workflows/ci.yml/badge.svg)](https://github.com/umpire274/rTimelog/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/umpire274/rTimelog)](https://github.com/umpire274/rTimelog/releases)
[![codecov](https://codecov.io/gh/umpire274/rTimelog/graph/badge.svg?token=5WPQF58D5Z)](https://codecov.io/gh/umpire274/rTimelog)

`rTimelog` is a simple, cross-platform **command-line tool** written in Rust to track daily working sessions, including
working position, start and end times, and lunch breaks.  
The tool calculates the expected exit time and the surplus of worked minutes.

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
- Display of the **total surplus** (sum of daily surplus/deficit) at the end of the `list` output.
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

For working position (O, R, H). You can specify the position in either uppercase or lowercase:

```bash
rtimelog list --pos O
rtimelog list --pos R
rtimelog list --pos H
```

---

Delete a session `id`

Remove a session by its `id`:

```bash
rtimelog del 1
```

---

## ⚙️ Configuration

When you run rtimelog init, a configuration file is created in your home directory:

- **Linux/macOS**: `$HOME/.rtimelog/rtimelog.conf`
- **Windows**: `%APPDATA%\rtimelog\rtimelog.conf`

### Example rtimelog.conf

```yaml
database: "/home/user/.rtimelog/rtimelog.sqlite"
default_position: "O"
min_duration_lunch_break: 30
max_duration_lunch_break: 90
```

**Parameters**:

- database: path of the SQLite database used by the application.
- default_position: default working position (O, R, C, H).
- min_duration_lunch_break: minimum lunch break in minutes (default: 30).
- max_duration_lunch_break: maximum lunch break in minutes (default: 90).

### Print the current configuration

You can print the absolute path of the configuration file and its contents with:

```bash
rtimelog conf --print
```

Output example:

```vbnet
📄 Config file: /home/user/.rtimelog/rtimelog.conf
database: "/home/user/.rtimelog/rtimelog.sqlite"
default_position: "O"
min_duration_lunch_break: 30
max_duration_lunch_break: 90
```

### Edit the configuration

You can edit the configuration file directly from the CLI:

- With the default editor of your platform:
  ```bash
  rtimelog conf --edit
  ```
- With a specific editor (e.g. `vi`):
  ```bash
  rtimelog conf --edit --editor vi
  ```

If the requested editor is not available on the platform, the file will be opened with the default system editor.

⚠️ On **Linux/macOS**, the default editor is taken from the `$EDITOR` environment variable.
If `$EDITOR` is not set, the system default editor will be used.

⚠️ On **Windows**, if you want to use an editor installed under `Program Files` (e.g. `Notepad++`), you must provide the
**absolute path** in quotes:

```ps
rtimelog conf --edit --editor "C:\Program Files\Notepad++\notepad++.exe"
```

---

## 🗄️ Database migrations

Starting from **v0.3.3**, `rTimelog` manages its own internal DB versioning:

- A dedicated table `schema_migrations` tracks all migrations applied.
- On every command execution (`init`, `add`, `list`, `del`), the application checks if the database schema is outdated.
- If pending migrations are found, they are applied automatically before continuing.

This ensures that older databases remain compatible with newer versions of the application without manual intervention.

---

## ⚠️ Notes

- Lunchtime is validated: minimum 30 minutes, maximum 90 minutes for Office (O) position.
- Holidays (H) ignore work and lunch logic, and are displayed as Holiday in purple background.
- The --db global option allows selecting a custom database per execution, useful for testing or separate datasets.

---

## 📊 Output example

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
 
                                                                                     -------------------------
                                                                                     Σ Total surplus:  -17 min
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
