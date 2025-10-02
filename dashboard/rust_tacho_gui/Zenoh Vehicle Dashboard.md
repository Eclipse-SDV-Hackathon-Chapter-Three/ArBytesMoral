<img src="https://r2cdn.perplexity.ai/pplx-full-logo-primary-dark%402x.png" style="height:64px;margin-right:32px"/>

# Zenoh Vehicle Dashboard

A real-time vehicle dashboard application built with Rust, displaying telemetry data received via Eclipse uProtocol and Zenoh transport protocol.

## Features

- Real-time message reception via Zenoh transport
- Interactive GUI dashboard built with egui/eframe
- Dynamic gauge visualization for numeric values
- Message history log with timestamps
- Multi-threaded architecture (Tokio runtime + GUI thread)
- Automatic value parsing from payload strings
- Dynamic gauge scaling based on incoming values


## Prerequisites

- Rust (stable) - Install from [rustup.rs](https://rustup.rs/)
- Zenoh router/infrastructure setup
- uProtocol compatible publisher


## Dependencies

```toml
[dependencies]
async-trait = "0.1.89"
clap = { version = "4.5.42", features = ["derive"] }
eframe = "^0.31"
egui_gauge = "0.1.5"
epaint = "^0.31"
parking_lot = "0.12.4"
serde_json = "1.0.145"
tokio = { version = "1.45.1", default-features = false, features = ["full"] }
up-rust = "0.7.1"
up-transport-zenoh = "0.8.0"
zenoh = "1.5.1"
```


## Building the project

Build the project:
```bash
cargo build --release
```


## Configuration

Create a Zenoh configuration file at `src/zenoh_config.json`. Example configuration:

```json
{
  "mode": "peer",
  "connect": {
    "endpoints": ["tcp/192.168.43.241:7447"]
  },
  "scouting": {
    "multicast": {
      "enabled": true,
      "address": "224.0.0.224:7446",
      "interface": "auto",
      "autoconnect": { "router": [], "peer": ["router", "peer"] },
      "listen": true
    },
    "gossip": {
      "enabled": true,
      "multihop": false,
      "autoconnect": { "router": [], "peer": ["router", "peer"] }
    }
  }
}
 
 
```

The subscriber listens for messages from:

- **Authority**: `vehicledataaccessor`
- **Entity ID**: `0x0000`
- **Version**: `2`
- **Resource ID**: `0x8002`

Modify these values in the source code (`main.rs`) if your publisher uses different URI parameters.

## Usage

### Running the Application

```bash
cargo run --release
```

Or run the compiled binary directly:

```bash
./target/release/<binary-name>
```

### Debug Logging

Enable detailed Zenoh logging:

```bash
export RUST_LOG=debug
cargo run
```

Or for specific components:

```bash
export RUST_LOG=zenoh=debug,up_transport_zenoh=debug
```


## Architecture

The application uses a multi-threaded architecture:

1. **Main Thread**: Runs the egui GUI event loop (required by winit/egui)
2. **Tokio Runtime Thread**: Handles async operations and Zenoh message reception
3. **Communication**: `tokio::sync::mpsc` unbounded channel for passing messages from Zenoh listener to GUI

### Data Flow

```
Publisher → Zenoh Transport → UListener.on_receive() 
    → mpsc channel → GUI update → Gauge display
```


## GUI Features

- **Large Gauge Display**: Shows current value with dynamic scaling
- **Message Log**: Displays last 20 messages with:
    - Source URI
    - Raw payload
    - Extracted numeric value
    - Time elapsed since reception
- **Automatic Updates**: GUI refreshes continuously to show new data


## Troubleshooting

### Messages Not Received

1. Verify Zenoh router is running and accessible
2. Check publisher is active and sending to correct topic
3. Verify URI filter matches publisher's source URI
4. Enable debug logging: `RUST_LOG=debug cargo run`

### GUI Not Starting

- Error about "EventLoop outside main thread": This is expected and was fixed by running GUI in main thread
- Ensure you're not running inside another GUI framework's event loop


### Build Errors

- Update dependencies: `cargo update`
- Clean build: `cargo clean && cargo build`


## Development

### Code Structure

- `main.rs`: Application entry point, setup, and GUI implementation
- `SubscriberListener`: Implements `UListener` trait for message reception
- `DashboardApp`: Implements `eframe::App` trait for GUI rendering
- `extract_number_from_payload()`: Parses integer values from payload strings


## License

[Specify your license here, e.g., Apache-2.0, MIT, etc.]

## Contributing

[Add contribution guidelines if applicable]

## Acknowledgments

- Built with [egui](https://github.com/emilk/egui) - immediate mode GUI framework
- Uses [Eclipse uProtocol](https://github.com/eclipse-uprotocol) for communication
- Powered by [Zenoh](https://zenoh.io/) transport protocol
<span style="display:none">[^1][^10][^2][^3][^4][^5][^6][^7][^8][^9]</span>

<div align="center">⁂</div>

[^1]: https://github.com/emilk/eframe_template

[^2]: https://github.com/slint-ui/slint-rust-template

[^3]: https://www.boringcactus.com/2025/04/13/2025-survey-of-rust-gui-libraries.html

[^4]: https://users.rust-lang.org/t/finding-a-rust-gui-that-is-easy-to-use/101950

[^5]: https://dev.to/davidedelpapa/rust-gui-introduction-a-k-a-the-state-of-rust-gui-libraries-as-of-january-2021-40gl

[^6]: https://whoisryosuke.com/blog/2022/create-a-markdown-editor-using-rust-and-react

[^7]: https://www.reddit.com/r/programming/comments/txlen9/next_level_readme_template/

[^8]: https://crates.io/crates/egui

[^9]: https://www.boringcactus.com/2020/08/21/survey-of-rust-gui-libraries.html

[^10]: https://invent.kde.org/sdk/rust-qt-binding-generator/-/blob/master/README.md

