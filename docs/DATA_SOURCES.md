# Data Sources Guide

This guide explains how to set up various ADS-B data sources for use with ADS-B TUI.

## What is ADS-B?

ADS-B (Automatic Dependent Surveillance-Broadcast) is a surveillance technology that allows aircraft to broadcast their position, altitude, speed, and other data. This data can be received by ground stations and shared with applications like ADS-B TUI.

## Hardware Requirements

To receive ADS-B data, you'll need:

- **ADS-B antenna** (1090 MHz)
- **ADS-B receiver** (RTL-SDR dongle or dedicated receiver)
- **Software decoder** (dump1090, readsb, etc.)

## Popular ADS-B Receivers

HTTP sources are allowed by default. Set `allow_http = false` (or omit `allow_http` entirely on HTTPS feeds) if you want to enforce HTTPS-only.

### 1. RTL-SDR with dump1090

**Hardware:**
- RTL-SDR dongle ($20-30)
- 1090 MHz antenna
- Raspberry Pi or computer

**Software Setup:**
```bash
# Install dump1090
sudo apt update
sudo apt install dump1090-fa

# Or compile from source
git clone https://github.com/flightaware/dump1090.git
cd dump1090
make
```

**Configuration for ADS-B TUI:**
```toml
url = "http://localhost:8080/data/aircraft.json"
refresh_secs = 1
allow_http = true
```

### 2. readsb (Modern Alternative)

**Installation:**
```bash
# Install readsb
sudo apt install readsb

# Configure readsb
sudo nano /etc/default/readsb
```

**readsb configuration:**
```
RECEIVER_OPTIONS="--device 0 --gain -10"
DECODER_OPTIONS="--max-range 360"
NET_OPTIONS="--net --net-heartbeat 60 --net-ro-size 1000 --net-ro-interval 1"
JSON_OPTIONS="--json-location-accuracy 2"
```

**ADS-B TUI config:**
```toml
url = "http://localhost:8080/data/aircraft.json"
refresh_secs = 1
allow_http = true
```

### 3. ADS-B Exchange Feeder

**Setup:**
1. Sign up at [ADS-B Exchange](https://www.adsbexchange.com/)
2. Install their feeder software
3. Configure sharing settings

**ADS-B TUI config:**
```toml
url = "https://adsbexchange.com/api/aircraft/json/"
refresh_secs = 2
# Note: May require API key for full access
```

## Online Data Sources

### ADS-B Exchange API

**Free access:**
```toml
url = "https://adsbexchange.com/api/aircraft/v2/lat/40.7128/lon/-74.0060/dist/250/"
refresh_secs = 5
# Set API key via env: ADSB_API_KEY=YOUR_API_KEY ADSB_API_KEY_HEADER=api-auth
```

**With API key (higher limits):**
```toml
url = "https://adsbexchange.com/api/aircraft/v2/lat/40.7128/lon/-74.0060/dist/250/?apiKey=YOUR_API_KEY"
refresh_secs = 1
# Set API key via env: ADSB_API_KEY=YOUR_API_KEY ADSB_API_KEY_HEADER=api-auth
```

### OpenSky Network

**Free access:**
```toml
url = "https://opensky-network.org/api/states/all"
refresh_secs = 10
# Note: Different JSON format, may require custom parsing
```

### FlightAware

**Requires API key:**
```toml
url = "https://flightaware.com/live/api/snapshot.rvt"
refresh_secs = 30
# Requires authentication headers
```

## Local Network Setup

### Multiple Receivers

If you have multiple receivers, you can combine their data:

```bash
# Run multiple dump1090 instances on different ports
dump1090 --device 0 --net --net-http-port 8080
dump1090 --device 1 --net --net-http-port 8081

# Use a tool like adsb-tools to combine feeds
```

### Docker Setup

**docker-compose.yml:**
```yaml
version: '3.8'
services:
  dump1090:
    image: flightaware/dump1090:latest
    devices:
      - /dev/bus/usb
    ports:
      - "8080:8080"
      - "30005:30005"
    command: --device 0 --net --net-http-port 8080
```

**ADS-B TUI config:**
```toml
url = "http://localhost:8080/data/aircraft.json"
refresh_secs = 1
```

## Data Format

ADS-B TUI expects JSON data in the format provided by dump1090:

```json
{
  "now": 1703123456,
  "aircraft": [
    {
      "hex": "a1b2c3",
      "flight": "UAL123",
      "r": "N12345",
      "t": "B737",
      "alt_baro": 35000,
      "lat": 40.7128,
      "lon": -74.0060,
      "track": 90,
      "speed": 500,
      "seen": 10
    }
  ]
}
```

### Required Fields

- `hex`: Aircraft ICAO address (hex)
- `r`: Aircraft registration (used for flags)
- `lat`: Latitude
- `lon`: Longitude
- `alt_baro`: Barometric altitude
- `track`: Ground track
- `speed`: Ground speed
- `seen`: Time since last message

### Optional Fields

- `flight`: Flight number/callsign
- `t`: Aircraft type
- `alt_geom`: Geometric altitude
- `gs`: Ground speed (alternative)
- `ias`: Indicated airspeed
- `tas`: True airspeed
- `mach`: Mach number
- `squawk`: Transponder code

## Troubleshooting Data Sources

### No Data Received

**Check antenna:**
- Ensure antenna is connected and positioned correctly
- Try different antenna locations
- Check for interference

**Check receiver:**
```bash
# Test RTL-SDR
rtl_test -t

# Check dump1090 status
sudo systemctl status dump1090-fa
```

**Check network:**
```bash
# Test local endpoint
curl http://localhost:8080/data/aircraft.json

# Check firewall
sudo ufw status
```

### Poor Data Quality

**Signal issues:**
- Improve antenna position/location
- Use better antenna
- Check for local interference

**Receiver settings:**
- Adjust gain settings
- Check sample rate
- Update firmware

### Performance Issues

**High CPU usage:**
- Use readsb instead of dump1090
- Reduce refresh rate
- Limit aircraft display range

**Network issues:**
- Check internet connection
- Reduce refresh rate for online sources
- Use local caching

## Advanced Setup

### Multiple Feeds

Combine data from multiple sources:

```bash
# Use adsb-tools to merge feeds
adsb-tools combine \
  --input http://receiver1:8080/data/aircraft.json \
  --input http://receiver2:8080/data/aircraft.json \
  --output http://localhost:9090/data/aircraft.json
```

### Custom Filtering

Filter aircraft by various criteria:

```toml
# Filter configuration
filter = "alt_baro > 10000 AND speed > 200"
```

### Data Export

Set up automatic data export:

```bash
# Export to CSV every 5 minutes
while true; do
  curl -s http://localhost:8080/data/aircraft.json | jq -r '.aircraft[] | @csv' >> aircraft_$(date +%Y%m%d).csv
  sleep 300
done
```

## Legal Considerations

- **ADS-B reception is legal** in most countries for personal use
- **Data sharing** may have restrictions depending on your location
- **Commercial use** often requires licenses or agreements
- Check local regulations before setting up public feeds

## Community Resources

- [ADS-B Community Forum](https://www.adsbcommunity.org/)
- [FlightAware Forums](https://discussions.flightaware.com/)
- [RTL-SDR Forums](https://www.rtl-sdr.com/)
- [ADS-B Exchange Discord](https://discord.gg/adsb)

## Getting Help

If you need help setting up your data source:

1. Check the [Issues](https://github.com/j4v3l/adsb-tui/issues) page
2. Ask in the [Discussions](https://github.com/j4v3l/adsb-tui/discussions) section
3. Provide details about your setup and any error messages
