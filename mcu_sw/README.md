# ThreadX MQTT IoT Sensor Node

## Overview

This embedded Rust application runs on the MxChip IoT DevKit, implementing a real-time IoT sensor node using Azure RTOS ThreadX. The system automatically measures environmental data and communicates via MQTT over WiFi. Simultaneously it receives a RGB value via MQTT and sets the integrated LED

## Features

### Hardware Support

- **STM32F4**
- **MxAz3166 IoT module** with WiFi connectivity
- **HTS221** temperature/humidity sensor
- **RGB LED** for visual feedback
- **OLED display** for status information
- **Push button** for user interaction


### Core Functionality

#### Multi-threaded Architecture

- **Measurement Thread**: Continuously reads temperature data from HTS221 sensor every 500 ms
- **Network Thread**: Handles WiFi connectivity, MQTT communication, and user interface updates
- **ThreadX RTOS**: Provides real-time task scheduling and inter-thread communication via queues


#### MQTT Communication

- **Publisher**: Automatically sends temperature readings to `mcu/temperature` topic
- **Subscriber**: Listens to `compute/color` topic for RGB LED control commands
- **Transport**: Uses Minimq library with custom ThreadX TCP/WiFi network stack

#### Display \& User Interface

- Real-time connection status updates
- Message counter display (sent/received)
- Last received/sent message preview
- Button-triggered emergency messaging


## System Architecture

```
┌─────────────────┐    ┌──────────────────┐
│ Measurement     │    │ Network          │
│ Thread          │───▶│ Thread           │
│ - Read sensor   │    │ - WiFi/MQTT      │
│ - Send to queue │    │ - Display update │
└─────────────────┘    │ - LED control    │
                       └──────────────────┘
```


## Configuration

### Network Settings

- **SSID**: `SDV_Chapter3-Team2`
- **Password**: `EclipseSDVC3`
- **MQTT Broker**: `192.168.43.241:1883`


### MQTT Topics

- **Publish**: `mcu/temperature` - Temperature readings in Celsius
- **Subscribe**: `compute/color` - RGB color commands


## Building and Deployment

This project requires:

- Rust with `no_std` embedded toolchain
- ThreadX RTOS integration
- STM32 HAL drivers
- Custom board support package for MxAz3166

The application uses `#![no_main]` and `#![no_std]` attributes for bare-metal embedded execution with ThreadX providing the runtime environment.

To build and run this project, navigate to the directory `threadx-rust/threadx-app/cross/app` and run `cargo run --release --target thumbv7em-none-eabihf --bin network`

## Key Technologies

- **Rust**: Memory-safe embedded systems programming
- **Azure RTOS ThreadX**: Real-time operating system
- **Minimq**: Lightweight MQTT client library
- **Embedded Graphics**: Display rendering
- **ThreadX-rs**: Rust bindings for ThreadX API

This system demonstrates modern embedded IoT development combining Rust's safety guarantees with real-time operating system capabilities for reliable sensor data collection and wireless communication.
<span style="display:none">[^1][^2][^3][^4][^5][^6][^7][^8][^9]</span>

<div align="center">⁂</div>

[^1]: https://docs.rs/homie-controller

[^2]: https://docs.rs/color-parser

[^3]: https://docs.esp-rs.org/std-training/03_5_3_mqtt.html

[^4]: https://crates.io/crates/mqtt-analyzer/dependencies

[^5]: https://github.com/seancroach/hex_color

[^6]: https://community.openhab.org/t/solved-mqtt-esphome-color-channel-integration/141824

[^7]: https://www.tinkerforge.com/en/doc/Software/API_Bindings_MQTT.html

[^8]: https://dev.to/emqx/how-to-use-mqtt-in-rust-5fne

[^9]: https://crates.io/crates/csscolorparser

