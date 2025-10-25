# Rusted Coffee PID

Rust implementation for a PID controller on a Rancilio Silvia espresso machine, running on ESP32-S3 with real-time temperature monitoring and brew timing.

**This is work in progress! For now only temperature monitoring and brew "simulation" are coded, more to come!**

## Features

- **Real-Time Display**: SSD1306 OLED showing live temperature and brew timer
- **Temperature Monitoring**: TSIC 306 sensor for continuous temperature reading
- **Interactive Brew Mode**: Push button to start timed brew cycles (0-60s)
- **Multi-Core Architecture**: Concurrent tasks for display, sensors, and input handling
- **Minimal Dependencies**: Custom async drivers for display and sensor

## Quick Start

### Building

```bash
laze build -b espressif-esp32-s3-devkitc-1
```

### Building and Flashing

```bash
laze build -b espressif-esp32-s3-devkitc-1 run
```

This will:
1. Compile the project in release mode
2. Flash to the connected ESP32-S3
3. Open serial monitor to see debug output

## Hardware

### Pin Configuration

| Component | GPIO | Protocol |
|-----------|------|----------|
| SSD1306 I2C SDA | 2 | I2C |
| SSD1306 I2C SCL | 0 | I2C |
| TSIC 306 Signal | 1 | ZACwire |
| Push Button | 9 | Digital Input (Active Low) |

### Connections

- **I2C Display**: SDA on GPIO2, SCL on GPIO0
- **Temperature Sensor**: ZACwire signal on GPIO1 with pull-up resistor
- **Button**: GPIO9 with hardware debouncing (active-low, connected to ground when pressed)

## Architecture

### Multi-Core Task Design

```
ESP32-S3 (Dual Core)
├─ display_task (I2C - GPIO2 SDA, GPIO0 SCL)
│  └─ SSD1306 OLED driver
│  └─ Brew state machine
│  └─ Display rendering
│
├─ sensor_task (GPIO1 - TSIC 306)
│  └─ Temperature monitoring
│  └─ Sends data via TEMP_CHANNEL
│
└─ button_task (GPIO9)
   └─ Button input polling
   └─ Sends events via BREW_BUTTON_CHANNEL
```

### Inter-Task Communication

- **TEMP_CHANNEL**: Temperature values from sensor_task to display_task
- **BREW_BUTTON_CHANNEL**: Button press events from button_task to display_task

## Display States

The SSD1306 OLED shows three states:

### IDLE State
- Label: "Ready"
- Shows: Large temperature display
- Action: Press button to start brew

### BREWING State
- Label: "Brewing..."
- Shows: Large timer (0-60 seconds), small temperature
- Duration: Auto-transitions to COMPLETE at 60 seconds

### COMPLETE State
- Label: "Complete!"
- Shows: Frozen timer at 60s, small temperature
- Action: Press button to return to IDLE

## Project Structure

```
src/
├── main.rs                    # Entry point and task spawner
├── counter_test.rs            # Display controller with UI
├── temperature_sensor.rs      # TSIC 306 sensor task
├── button.rs                  # Button input handler
├── brew_controller.rs         # Brew state machine
├── comm.rs                    # Inter-task channels
├── pins.rs                    # GPIO configuration
└── drivers/
    ├── ssd1306_async/         # Custom async SSD1306 driver
    └── tsic306_async/         # TSIC 306 sensor driver
```

## Prerequisites

- Rust toolchain with ESP32-S3 target support
- `laze` build tool (Ariel OS project builder)
- `espflash` for flashing to device
- Connected ESP32-S3 DevKitC-1 board

## Troubleshooting

### Build Fails
- Ensure you're in the project directory
- Run `laze build -b espressif-esp32-s3-devkitc-1 clean` to clear cache

### Display Not Showing
- Check I2C connections (GPIO0 for SCL, GPIO2 for SDA)
- Verify SSD1306 address is 0x3C
- Confirm I2C frequency is 100kHz

### Temperature Not Reading
- Check TSIC 306 connection on GPIO1
- Verify pull-up resistor is present (typically 4.7k-10k ohms)
- Check ZACwire signal timing

### Button Not Responding
- Verify button is connected to GPIO9
- Confirm hardware debouncing capacitor value
- Ensure button pulls GPIO9 to ground when pressed

## License

MIT License
