# Rusted Coffee PID - Build & Run Guide

## Overview

Your ESP32-S3 Coffee PID controller now has:
- ✅ **OLED Display**: Live 60-second timer and temperature readings
- ✅ **Temperature Sensor**: TSIC 306 ZACwire continuous reading
- ✅ **HTTP Server**: RESTful API to access temperature and status via WiFi

## Building the Project

### Quick Build with WiFi

The project requires WiFi credentials to build. These are stored in `.env` file:

```bash
# Edit .env with your WiFi credentials
nano .env

# Then build with:
cd /Users/facosta/git/kYc0o/rusted-coffee-pid
export CONFIG_WIFI_NETWORK="YourSSID"
export CONFIG_WIFI_PASSWORD="YourPassword"
laze build -b espressif-esp32-s3-devkitc-1 run
```

### One-Liner Build Command

```bash
cd /Users/facosta/git/kYc0o/rusted-coffee-pid && \
export CONFIG_WIFI_NETWORK="NoTeMetasConMiCucu-2.4GHz" && \
export CONFIG_WIFI_PASSWORD="Ch0l3Dian@yElD0n" && \
laze build -b espressif-esp32-s3-devkitc-1 run
```

### Using .env File (Easier)

You can source the .env file for development:

```bash
cd /Users/facosta/git/kYc0o/rusted-coffee-pid

# Load environment variables manually before calling laze
export $(cat .env | grep -v '^#' | xargs)

# Then build
laze build -b espressif-esp32-s3-devkitc-1 run
```

## Project Structure

```
rusted-coffee-pid/
├── .env                          # WiFi credentials (edit this!)
├── Cargo.toml                    # Dependencies
├── laze-project.yml              # Build configuration with network module
├── src/
│   ├── main.rs                   # HTTP server spawner & app builder
│   ├── counter_test.rs           # 60s timer + display task (autostart with peripherals)
│   ├── temperature_sensor.rs     # TSIC 306 sensor reader (spawned from counter_test)
│   ├── routes.rs                 # HTTP endpoints
│   ├── comm.rs                   # Shared temperature channel
│   ├── pins.rs                   # GPIO & I2C definitions
│   ├── oled_ssd1306.rs           # Display driver
│   └── drivers/
│       ├── ssd1306_async/        # Async SSD1306 implementation
│       └── tsic306_async/        # Async TSIC 306 reader
└── static/
    └── index.html                # Web dashboard
```

## Features & Functionality

### Display (OLED - SSD1306)
- **Top**: "Coffee PID" title
- **Center**: 60-second countdown timer (large font)
- **Bottom**: Current temperature (small font)
- **Cycle**: Auto-restarts after reaching 60 seconds

### Temperature Sensor (TSIC 306 on GPIO1)
- Reads every 100ms
- Sends to `TEMP_CHANNEL` (embassy_sync channel)
- Displays on OLED in real-time
- Exposed via HTTP `/temperature` endpoint

### HTTP Server (Port 80)
- **2 concurrent connections** (WEB_TASK_POOL_SIZE = 2)
- Uses `picoserve` library
- Automatically connects to WiFi via DHCP

### Available Endpoints

1. **GET /** - Static HTML dashboard
   - Auto-refreshing every 5 seconds
   - Shows current temperature
   - Shows device status

2. **GET /temperature** - JSON temperature data
   ```json
   {
     "temperature": 23.5,
     "unit": "°C",
     "timestamp": 0
   }
   ```

3. **GET /status** - Server status JSON
   ```json
   {
     "device": "ESP32-S3 Coffee PID",
     "status": "running",
     "uptime_ms": 12345,
     "http_port": 80
   }
   ```

## Network Configuration

### Automatic (DHCP)
By default, the device gets an IP address automatically from your WiFi router.

1. Flash the device
2. Connect to `NoTeMetasConMiCucu-2.4GHz` WiFi
3. Check your router's DHCP client list for the device IP
4. Access at `http://<device-ip>/`

### Static IP (Optional)

Edit `.env` and uncomment:

```env
CONFIG_NET_IPV4_STATIC_ADDRESS=192.168.1.100
CONFIG_NET_IPV4_STATIC_CIDR_PREFIX_LEN=24
CONFIG_NET_IPV4_STATIC_GATEWAY_ADDRESS=192.168.1.1
```

Then rebuild with `export $(cat .env | grep -v '^#' | xargs)`.

## Hardware Mapping

| Component | GPIO | Purpose |
|-----------|------|---------|
| I2C SDA   | GPIO2 | OLED Display |
| I2C SCL   | GPIO0 | OLED Display |
| TSIC 306  | GPIO1 | Temperature Sensor |

## Flashing to Device

When running with `laze build ... run`, you'll be prompted to select a serial port:

```bash
Detected 2 serial ports
Please select a port:
> /dev/tty.usbserial-XXXXXX
  /dev/cu.usbserial-XXXXXX
```

Select the appropriate port and the device will flash automatically.

## Testing the HTTP Server

### From Command Line

```bash
# Get device IP from router or use mDNS
DEVICE_IP=192.168.1.100

# Test endpoints
curl http://$DEVICE_IP/
curl http://$DEVICE_IP/temperature
curl http://$DEVICE_IP/status
```

### From Browser

Simply open `http://<device-ip>/` in your browser to see the live dashboard.

## Troubleshooting

### Build fails with "CONFIG_WIFI_NETWORK not set"
- **Fix**: Make sure to export the environment variables before building
- ```bash
  export CONFIG_WIFI_NETWORK="YourSSID"
  export CONFIG_WIFI_PASSWORD="YourPassword"
  ```

### Build fails with "impl_trait_in_assoc_type is unstable"
- **Fix**: This requires a nightly Rust feature (already added to main.rs)
- The project uses `#![feature(impl_trait_in_assoc_type)]`

### Device not connecting to WiFi
- **Fix**: Check the WiFi SSID and password in `.env`
- Verify the network is 2.4GHz (ESP32 doesn't support 5GHz)
- Check WiFi credentials are correct in the environment

### Can't access HTTP server
- **Fix**: 
  1. Verify device is connected to WiFi
  2. Check device IP from router's DHCP list
  3. Ensure firewall allows port 80 traffic
  4. Try accessing via `http://` not `https://`

## Development Notes

- **Display & Sensor**: Run together as `counter_test` task (autostart with peripherals)
- **HTTP Server**: Runs in background via `web_task` pool (2 concurrent connections)
- **Temperature**: Shared via `embassy_sync::Channel` in `comm.rs` module
- **No Console Output**: All debug logging removed for production use

## Next Steps

Possible enhancements:
- Add `/control` endpoint to adjust PID parameters
- Add HTTPS support (requires certificates)
- Add WebSocket for real-time updates
- Implement authentication
- Add more sensors (pressure, flow rate, etc.)

---

**Ready to build?**

```bash
cd /Users/facosta/git/kYc0o/rusted-coffee-pid && \
export CONFIG_WIFI_NETWORK="NoTeMetasConMiCucu-2.4GHz" && \
export CONFIG_WIFI_PASSWORD="Ch0l3Dian@yElD0n" && \
laze build -b espressif-esp32-s3-devkitc-1 run
```
