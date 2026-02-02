# Changelog

All notable changes to ADS-B TUI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Logging system** with configurable log levels, file output, and structured logging
- **Route backoff logic** and error handling for API rate limiting
- **Total aircraft rate tracking** with decay logic for improved rate calculations
- **Enhanced project configuration** and improved justfile commands
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

### Changed
- Moved FLAG column to first position in aircraft table
- Improved Cargo.toml with comprehensive metadata
- Enhanced error handling and code quality
- Better project organization and documentation

### Fixed
- Flag display logic to properly handle ICAO prefix variations
- Various code quality improvements from clippy

## [0.1.0] - 2024-01-XX

### Added
- Initial release of ADS-B TUI
- Real-time aircraft tracking with terminal UI
- Support for dump1090 and similar ADS-B receivers
- Aircraft table with customizable columns
- Favorites system for tracking specific aircraft
- Data export functionality (CSV/JSON)
- Route information display
- Basic configuration system
- Cross-platform support (Windows, macOS, Linux)

### Features
- Live aircraft position, altitude, speed, and heading
- Distance and bearing calculations
- Aircraft trail visualization
- Filter and search capabilities
- Keyboard shortcuts for navigation
- Smooth scrolling and responsive UI
