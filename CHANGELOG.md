# Changelog

# Changelog

# Changelog

## [0.2.1] - 2025-09-15
### Added
- Support in `init` command for initializing a new database in:
  - an absolute path
  - directories containing spaces in their names

### Changed
- Updated `list` command: now shows the **expected end time** even when only the start time is provided for a given date
- Updated integration tests for the new version v0.2.1

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
