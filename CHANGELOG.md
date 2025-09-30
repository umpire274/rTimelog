# Changelog

# [0.3.4] - 2025-09-30

### Added

- Print the record inserted or updated when invoking the `add` command (the command now displays only the affected record).
- Configuration files for GitHub Copilot: `copilot-custom.json` (machine-readable) and `copilot-custom.md` (human-readable documentation).
- Bump project version to `v0.3.4` and update dependencies as required.

### Changed

- Updated dependencies and version metadata for the `v0.3.4` release.

---


# [0.3.3] - 2025-09-18

### Added

- New internal DB versioning system to handle schema evolution.
- New table schema_migrations to record each migration applied.
- Automatic check and execution of pending migrations every time a command is run.
- Automatic configuration file migration:
    - Adds missing parameters min_duration_lunch_break (default 30)
    - and max_duration_lunch_break (default 90)

### Changed

- The logic for expected exit time now uses configurable lunch break limits from the configuration file instead of
  hardcoded values.
- Improved conf --edit command:
    - If the requested editor does not exist, the application now falls back to the default editor ($EDITOR/nano on
      Linux/macOS, notepad on Windows) instead of panicking.

---

# [0.3.2] - 2025-09-17

### Added

- New command del to delete a work session by id from the work_sessions table.
- New working position C = On-Site (Client).
- Utility function to map working positions (O, R, C, H) into descriptive, colorized labels.
- Unit test for the new utility function.
- Integration tests for:
    - del command (successful and unsuccessful cases).
    - describe_position function.

### Changed

- Output of the list command updated:
- Supports the new working position C=On-Site (Client).
- Displays colorized working positions for better readability.
- Reformatted integration test outputs for consistency.
- Updated SQL in init command to support the new position C.
- Introduced migration function for release 0.3.2.

---

# [0.3.1] - 2025-09-17

### Added

- New global option `--pos` in the `list` command to filter sessions by working position:
    - `O` = Office
    - `R` = Remote
    - `H` = Holiday
- A function `make_separator` and `print_separator` in `utils.rs` to generate aligned separators with custom character,
  width, and alignment.
- Unit tests for `make_separator`.
- Integration test for the new `--pos` option of `list` command.
- Display of the **total surplus** (sum of daily surpluses) at the end of the `list` output.

### Changed

- Improved the output formatting of the `list` command, including:
    - aligned `Lunch` time using `HH:MM` or padded `-`
    - cleaner separator handling with the new utility functions.

---

# [0.3.0] - 2025-09-16

### Added

- New parameter `working_time` in application configuration file to define the daily working duration.
- Support for new position `H` (Holiday) in work sessions, with purple highlighted visualization in `list` command.
- Database migration mechanism to automatically upgrade schema when needed.
- Utility snippets in Rust to convert `NaiveDate` and `NaiveDateTime` to/from ISO 8601 text strings for SQLite
  compatibility.

### Changed

- Updated all calculation logic for expected exit time to use the configurable `working_time` parameter.
- Changed the visualization of lunchtime from number of minutes to `HH:MM` notation.
- Updated integration tests to validate:
    - usage of the new `working_time` parameter
    - new position `H` (Holiday)
    - DB migration functionality

---

## [0.2.5] - 2025-09-16

### Added

- New `conf` command to handle the configuration file
- `--print` option for `conf` to print the current configuration file
- `--edit` option for `conf` to edit the configuration file
- `--editor` option for `conf`, to be used with `--edit`, to specify which editor to use (supports `vim`, `nano`, or any
  custom path)
- Help messages for the new `conf` command and its options

### Changed

- Separated command implementations from `main.rs` into a new `commands.rs` source file

### Fixed

- Removed a stray debug print line

---

## [0.2.1] - 2025-09-15

### Added

- Support in `init` command for initializing a new database in:
    - an absolute path
    - directories containing spaces in their names

### Changed

- Updated `list` command: now shows the **expected end time** even when only the start time is provided for a given date
- Updated integration tests for the new version v0.2.1

### Fixed

- Prevented production config (`rtimelog.conf`) from being overwritten during integration tests by introducing `--test`
  global flag
- Ensured consistent DB path resolution when using `--db` together with `--test`

---

## [0.2.0] - 2025-09-14

### Added

- Creation of a configuration file in the user home (depending on platform) with:
    - DB filename
    - Default working position (`O`)
- New column `position` in SQLite DB to identify the working position of the day:
    - `O` = Office
    - `R` = Remote
- New options for `add` command:
    - `--pos` add the working position of the day, O = Office, R = Remote
    - `--in` add the start hour of work
    - `--lunch` add the duration of lunch
    - `--out` add the end hour of work
- Added global option `--db` to specify:
    - a DB name (created under rTimelog config directory)
    - or an absolute DB path
- Added a message when the DB is empty (`⚠️ No recorded sessions found`)

### Changed

- Reorganized the output of the `list` command
- Updated integration tests for new DB column `position`
- Updated the logic for opening the connection to the DB file
- Updated integration tests to use `--db` option

### Notes

- Previous intermediate changes introduced `--name` and config file handling,
  but they have been replaced by the new global `--db` approach for consistency.

---

## [v0.1.2] - 2025-09-12

### Added

- Added functionality to search records by year (`yyyy`) or year-month (`yyyy-mm`) using option `--period`.
- Added explicit `+` sign for positive surplus minutes.

### Changed

- Updated integration tests to cover new functionalities.

## [0.1.1] - 2025-09-12

### Added

- New workflow: `release.yml` for automated releases
- New workflow: `ci.yml` for multi-platform build and test (Linux, Windows, macOS Intel & ARM)
- Added Unit Test for Logic and Integration between DB and Logic

### Changed

- Updated `README.md` with badges and new documentation
- Fixed formatting issues detected by `cargo fmt`

### Removed

- Deleted obsolete workflow `.github/workflows/rust.yml`

## [0.1.0] - 2025-09-12

### Added

- Create CHANGELOG.md
- Create LICENSE
- Create rust.yml
- Create README.md
- Set origin language in English
- If the date parameter is empty, assume the current date
- Initial version of the project
