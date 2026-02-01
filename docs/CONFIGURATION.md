# Configuration Guide

This guide explains all configuration options available in ADS-B TUI.

## Configuration File Location

ADS-B TUI looks for configuration in the following locations (in order of priority):

1. `adsb-tui.toml` in the current working directory
2. `adsb-tui.toml` in the user's config directory
3. Default values (built into the application)

## Basic Configuration

Create a file named `adsb-tui.toml` with your settings:

```toml
# ADS-B data source URL (required)
url = "http://your-adsb-receiver:8080/data/aircraft.json"

# Refresh interval in seconds (0 = fetch once)
refresh_secs = 1

# Allow insecure HTTPS connections
insecure = false
```

## Complete Configuration Reference

### Data Source Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `url` | string | *required* | URL of your ADS-B data source |
| `refresh_secs` | number | 0 | How often to fetch new data (0 = fetch once) |
| `insecure` | boolean | false | Allow self-signed SSL certificates |
| `stale_secs` | number | 60 | Mark aircraft as stale after this many seconds |

### Data Quality Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `low_nic` | number | 5 | Minimum Navigation Integrity Category |
| `low_nac` | number | 8 | Minimum Navigation Accuracy Category |

### Display Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `flags_enabled` | boolean | true | Show country flags for aircraft |
| `trail_len` | number | 6 | Length of aircraft trail lines |
| `radar_range_nm` | number | 200.0 | Radar max range in nautical miles |
| `radar_aspect` | number | 1.0 | Radar Y-axis scale factor for aspect compensation |
| `radar_renderer` | string | "canvas" | Radar renderer ("canvas", "ascii") |
| `radar_labels` | boolean | false | Show labels above radar blips (full radar layout) |
| `radar_blip` | string | "dot" | Blip style ("dot", "block", "plane") |
| `ui_fps` | number | 60 | UI refresh rate in frames per second |
| `smooth_mode` | boolean | true | Enable smooth scrolling |
| `altitude_trend_arrows` | boolean | true | Show altitude trend arrows |
| `track_arrows` | boolean | true | Show track direction arrows |

### Route Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `route_enabled` | boolean | true | Enable flight route display |
| `route_base` | string | "https://api.adsb.lol" | Route data API base URL |
| `route_mode` | string | "routeset" | Route data mode |
| `route_path` | string | "tar1090/data/routes.json" | Route data path |
| `route_ttl_secs` | number | 0 | Route cache time-to-live |
| `route_refresh_secs` | number | 0 | Route refresh interval |
| `route_batch` | number | 20 | Batch size for route requests |
| `route_timeout_secs` | number | 6 | Route request timeout |

### File Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `favorites_file` | string | "adsb-favorites.txt" | Path to favorites file |
| `filter` | string | "" | Aircraft filter expression |

### UI Layout Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `layout` | string | "full" | UI layout mode ("full", "compact", "radar") |
| `theme` | string | "default" | Color theme |

### Performance Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `column_cache` | boolean | true | Cache column calculations |
| `rate_window_ms` | number | 500 | Rate calculation window |
| `rate_min_secs` | number | 0.4 | Minimum rate interval |

### Notification Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `notify_radius_mi` | number | 10.0 | Notification radius in miles |
| `overpass_mi` | number | 0.5 | Overpass distance threshold |
| `notify_cooldown_secs` | number | 120 | Notification cooldown period |

### Location Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `site_lat` | number | *required* | Your location latitude |
| `site_lon` | number | *required* | Your location longitude |
| `site_alt_m` | number | *required* | Your location altitude in meters |
| `log_enabled` | boolean | false | Enable logging to file |
| `log_level` | string | "info" | Logging level (trace/debug/info/warn/error) |
| `log_file` | string | "adsb-tui.log" | Log output file path |

## Example Configurations

### Basic Local Setup

```toml
url = "http://localhost:8080/data/aircraft.json"
refresh_secs = 2
flags_enabled = true
site_lat = 40.7128
site_lon = -74.0060
site_alt_m = 10.0
```

### ADS-B Exchange Setup

```toml
url = "https://adsbexchange.com/api/aircraft/json/"
refresh_secs = 1
insecure = false
api_key = "YOUR_API_KEY"
api_key_header = "api-auth"
log_enabled = true
log_level = "debug"
log_file = "adsb-tui.log"
flags_enabled = true
route_enabled = true
site_lat = 51.5074
site_lon = -0.1278
site_alt_m = 25.0
```

### High-Performance Setup

```toml
url = "http://localhost:8080/data/aircraft.json"
refresh_secs = 0.5
ui_fps = 120
smooth_mode = true
column_cache = true
flags_enabled = true
route_enabled = true
site_lat = 37.7749
site_lon = -122.4194
site_alt_m = 50.0
```

## Command Line Overrides

You can override configuration values using command line arguments:

```bash
# Override URL and refresh rate
adsb-tui --url "http://example.com/data.json" --refresh-secs 5

# Enable debug logging to file
adsb-tui --log --log-level debug --log-file adsb-tui.log

# Enable insecure connections
adsb-tui --insecure

# Set custom config file
adsb-tui --config my-config.toml
```

## Environment Variables

ADS-B TUI respects some environment variables:

- `ADSBTUI_CONFIG` - Path to configuration file
- `ADSBTUI_URL` - Data source URL (overrides config)
- `RUST_LOG` - Logging level (for debugging)

## Troubleshooting

### Common Issues

**"Connection refused" errors:**
- Check that your ADS-B receiver is running
- Verify the URL in your configuration
- Ensure firewall allows connections

**"No aircraft displayed":**
- Check `low_nic` and `low_nac` settings
- Verify your location coordinates are correct
- Check `stale_secs` setting

**Performance issues:**
- Increase `refresh_secs` to reduce load
- Decrease `ui_fps` if UI is laggy
- Enable `column_cache` for better performance

### Debug Mode

Enable debug logging to troubleshoot issues:

```bash
RUST_LOG=debug adsb-tui
```

This will show detailed information about data fetching, parsing, and UI updates.
