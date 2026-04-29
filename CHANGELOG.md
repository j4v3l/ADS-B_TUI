# Changelog

All notable changes to ADS-B TUI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-04-29

### Added
- Runtime radar viewport controls: `+`/`=` zoom in, `-` zooms out, and Shift+arrow keys pan the radar/feed center.
- Dynamic feed URL templates with `{lat}`, `{lon}`, `{range_nm}`, `{range}`, and `{zoom}` placeholders for point feeds that follow the active radar viewport.
- 2D radar-only arrow navigation that selects the nearest plotted aircraft north, south, east, or west of the current target.
- Selected aircraft rendering on the radar canvas and ASCII fallback so the current target remains visible even when labels are disabled.
- Quit confirmation modal for `q`, with explicit confirm/cancel controls.

### Changed
- Separated receiver/home site location from mutable radar/feed center so distance, bearing, overpass, and notifications keep using the configured site.
- Updated radar help, footer, legend, README, configuration docs, and data-source docs for viewport controls and dynamic URL templates.
- Tightened radar and table display styling for selection, compact status text, and modal readability.

### Fixed
- Remote point-feed fetching now updates promptly after radar pan/zoom by replacing source URLs and interrupting fetcher sleeps.
- Radar selected-target display now uses first-class selected point state instead of relying only on labels or a thin overlay marker.
- Selection panels now distinguish selected aircraft that are outside the current radar range or missing position data.
- Clippy warnings in runtime key handling were resolved.

## [0.1.0] - 2026-02-09

### Added
- **Logging system** with configurable log levels, file output, and structured logging
- **Route backoff logic** and error handling for API rate limiting
- **Total aircraft rate tracking** with decay logic for improved rate calculations
- **Enhanced project configuration** and improved justfile commands
- Pre-commit hook setup for `cargo fmt` and `cargo clippy` to align local checks with CI
- VS Code workspace configuration for easier contributor onboarding
- Country flag display for aircraft based on ICAO registration prefixes
- Comprehensive flag mapping for 80+ countries
- Configurable flag visibility (`flags_enabled` option)
- Beginner-friendly documentation and setup guides
- CI/CD pipeline with cross-platform builds (Windows, macOS, Linux)
- Security auditing and dependency checking
- Integration tests and expanded unit test coverage
- Professional project structure with CONTRIBUTING.md, LICENSE, etc.
- Configuration guide and data sources documentation
- Development workflow improvements with enhanced justfile
- Initial release of ADS-B TUI
- Real-time aircraft tracking with terminal UI
- Support for dump1090 and similar ADS-B receivers
- Aircraft table with customizable columns
- Favorites system for tracking specific aircraft
- Data export functionality (CSV/JSON)
- Route information display
- Basic configuration system
- Cross-platform support (macOS, Linux)

### Changed
- Moved FLAG column to first position in aircraft table
- Refined GitHub issue templates (restored directory, default assignee, clearer examples)
- Improved Cargo.toml with comprehensive metadata
- Enhanced error handling and code quality
- Better project organization and documentation
- Reorganized runtime tests to satisfy lint ordering and reduce noise

### Fixed
- Flag display logic to properly handle ICAO prefix variations
- Various code quality improvements from clippy
- Resolved remaining clippy lints (app/radar/runtime) including replacing manual parity math in UI timing

### Features
- Live aircraft position, altitude, speed, and heading
- Distance and bearing calculations
- Aircraft trail visualization
- Filter and search capabilities
- Keyboard shortcuts for navigation
- Smooth scrolling and responsive UI
