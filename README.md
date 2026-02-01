# ADS-B TUI

A modern, fast, and user-friendly terminal interface for tracking aircraft using ADS-B data. Display real-time flight information in a beautiful table format with country flags, routes, and more.

[![Build Status](https://github.com/j4v3l/adsb-tui/workflows/CI/badge.svg)](https://github.com/j4v3l/adsb-tui/actions)
[![License](https://img.shields.io/badge/License-ADS--B--TUI--Non--Commercial-blue.svg)](https://github.com/j4v3l/adsb-tui/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![MSRV](https://img.shields.io/badge/MSRV-1.70-blue.svg)](https://github.com/j4v3l/adsb-tui)
[![Crates.io](https://img.shields.io/crates/v/adsb-tui.svg)](https://crates.io/crates/adsb-tui)
[![Downloads](https://img.shields.io/github/downloads/j4v3l/adsb-tui/total.svg)](https://github.com/j4v3l/adsb-tui/releases)
[![GitHub Stars](https://img.shields.io/github/stars/j4v3l/adsb-tui.svg)](https://github.com/j4v3l/adsb-tui/stargazers)
[![Discord](https://img.shields.io/discord/1234567890?label=Discord)](https://discord.gg/adsb-tui)
[![Matrix](https://img.shields.io/badge/Matrix-%23adsb--tui%3Amatrix.org-blue)](https://matrix.to/#/#adsb-tui:matrix.org)

> **Note**: Discord badge uses placeholder server ID. Update with your actual Discord server ID.

![adsb-tui screenshot](https://via.placeholder.com/800x400/1a1a1a/ffffff?text=ADS-B+TUI+Screenshot)

## ✨ Features

- **Real-time aircraft tracking** - Live ADS-B data from your receiver
- **Beautiful terminal UI** - Modern interface using Ratatui
- **Country flags** - Visual identification by aircraft registration
- **Flight routes** - Origin/destination information
- **Customizable columns** - Show/hide columns as needed
- **Favorites system** - Mark and track specific aircraft
- **Export functionality** - Save data to CSV/JSON
- **Cross-platform** - Works on Windows, macOS, and Linux

## 🚀 Quick Start

### Prerequisites

- **Rust** (latest stable version recommended)
- **ADS-B receiver** or access to an ADS-B data feed

### Installation

#### Option 1: Download Pre-built Binary

Download the latest release for your platform from the [Releases](https://github.com/j4v3l/adsb-tui/releases) page.

#### Option 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/j4v3l/adsb-tui.git
cd adsb-tui

# Build in release mode
cargo build --release

# The binary will be in target/release/adsb-tui
```

### Basic Usage

1. **Set up your data source**: Edit `adsb-tui.toml` to point to your ADS-B data URL
2. **Run the application**:
   ```bash
   ./adsb-tui
   ```

## 📖 Configuration

Create an `adsb-tui.toml` file in the same directory as the binary:

```toml
# ADS-B data source URL
url = "http://your-adsb-receiver/data/aircraft.json"

# Refresh interval in seconds (0 = fetch once)
refresh_secs = 1

# Allow insecure HTTPS connections
insecure = false

# Data staleness threshold in seconds
stale_secs = 60

# Enable logging
log_enabled = true
log_level = "info"
log_file = "adsb-tui.log"
```

# Minimum acceptable NIC (Navigation Integrity Category)
low_nic = 5

# Minimum acceptable NACp (Navigation Accuracy Category)
low_nac = 8

# Trail length for aircraft tracks
trail_len = 6

# Enable country flags
flags_enabled = true

# UI refresh rate in FPS
ui_fps = 60

# Smooth scrolling mode
smooth_mode = true
```

### Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `url` | ADS-B data source URL | Required |
| `refresh_secs` | Data refresh interval (0 = fetch once) | 0 |
| `insecure` | Allow self-signed certificates | false |
| `stale_secs` | Mark data as stale after this many seconds | 60 |
| `low_nic` | Minimum NIC value to display | 5 |
| `low_nac` | Minimum NACp value to display | 8 |
| `trail_len` | Aircraft trail length | 6 |
| `flags_enabled` | Show country flags | true |
| `ui_fps` | UI refresh rate | 60 |
| `smooth_mode` | Enable smooth scrolling | true |
| `log_enabled` | Enable logging to file | false |
| `log_level` | Logging level (trace/debug/info/warn/error) | "info" |
| `log_file` | Log output file path | "adsb-tui.log" |

## 🎮 Controls

| Key | Action |
|-----|--------|
| `↑/↓` | Navigate aircraft list |
| `←/→` | Scroll table horizontally |
| `f` | Toggle favorite for selected aircraft |
| `c` | Toggle column visibility |
| `e` | Export current data |
| `q` | Quit application |
| `?` | Show help |

## � Logging

ADS-B TUI includes comprehensive logging functionality for debugging and monitoring:

### Quick Start
```bash
# Enable logging with default settings
adsb-tui --log-enabled

# Enable debug logging to file
adsb-tui --log-enabled --log-level debug --log-file adsb-tui.log
```

### Log Levels
- `trace` - Most verbose, includes all internal operations
- `debug` - Detailed debugging information
- `info` - General information (default)
- `warn` - Warning messages
- `error` - Error messages only

### Environment Variables
```bash
# Override log level via environment
RUST_LOG=debug adsb-tui

# Or use ADS-B TUI specific variables
ADSB_LOG_ENABLED=1 ADSB_LOG_LEVEL=debug adsb-tui
```

### Log Output
Logs include timestamps, log levels, and structured information about:
- Network requests and responses
- Data parsing and processing
- UI state changes
- Error conditions and recovery
- Performance metrics

## �🛠️ Development

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Git

### Setup

```bash
# Clone the repository
git clone https://github.com/j4v3l/adsb-tui.git
cd adsb-tui

# Install development dependencies
cargo install cargo-watch cargo-edit cargo-audit

# Run in development mode with auto-reload
cargo watch -x run
```

### Development Commands

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test

# Run all checks
just check

# Build release binary
just build-release
```

### Project Structure

```
src/
├── main.rs      # Application entry point
├── app.rs       # Main application logic and state
├── ui.rs        # Terminal user interface
├── config.rs    # Configuration parsing
├── model.rs     # Data models
├── net.rs       # Network fetching
├── routes.rs    # Flight route handling
├── export.rs    # Data export functionality
├── storage.rs   # File storage operations
└── watchlist.rs # Watchlist management
```

## 📊 Data Sources

ADS-B TUI works with any ADS-B receiver that provides JSON data in the format used by [dump1090](https://github.com/flightaware/dump1090) or similar software.

### Popular ADS-B Receivers

- **dump1090** - The original ADS-B decoder
- **readsb** - Modern, fast ADS-B decoder
- **ADS-B Exchange** - Global flight tracking network
- **FlightAware** - Commercial flight tracking service

### Sample Data URLs

```
# Local dump1090
http://localhost:8080/data/aircraft.json

# ADS-B Exchange (requires API key)
https://adsbexchange.com/api/aircraft/json/

# FlightAware (requires API key)
https://flightaware.com/live/api/
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Quick Start for Contributors

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes
4. Add tests for new functionality
5. Run the test suite: `cargo test`
6. Format your code: `cargo fmt`
7. Submit a pull request

## � Security

For security-related concerns, please see our [Security Policy](SECURITY.md).

## 📄 License

This project is licensed under the ADS-B TUI Non-Commercial License - see the [LICENSE](LICENSE) file for details.

**Important**: This license allows personal use with attribution but prohibits commercial use. For commercial licensing inquiries, please contact the author.

## 🙏 Acknowledgments

- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [ADS-B Community](https://www.ads-b.com/) - For the amazing aviation data
- [Rust Community](https://www.rust-lang.org/) - For the excellent programming language

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/j4v3l/adsb-tui/issues)
- **Discussions**: [GitHub Discussions](https://github.com/j4v3l/adsb-tui/discussions)
- **Documentation**: [Wiki](https://github.com/j4v3l/adsb-tui/wiki)

---

**Happy flying! ✈️**
