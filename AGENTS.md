# AGENTS.md - Working in Lan Mouse

This document helps agents understand the Lan Mouse codebase for effective development.

## Project Overview

Lan Mouse is a cross-platform software KVM switch (mouse and keyboard sharing over LAN) written in Rust. It uses a client-server architecture where each instance can both send and receive input events.

**Key characteristics**:
- Rust workspace with multiple crates
- Async/await with tokio runtime
- Cross-platform support (Linux, Windows, macOS, X11, Wayland)
- Multiple backend implementations for input capture/emulation
- GTK4/libadwaita frontend (optional)
- DTLS-encrypted network communication

## Workspace Structure

```
lan-mouse/              # Main crate (service + application)
├── src/                # Core service logic
├── Cargo.toml          # Workspace root + main crate
└── build.rs            # shadow-rs build script

input-capture/          # Input capture backends (layer-shell, libei, Windows, macOS, X11)
├── src/
└── Cargo.toml          # Platform-specific features

input-emulation/        # Input emulation backends (wlroots, libei, Windows, macOS, X11, xdp)
├── src/
└── Cargo.toml          # Platform-specific features

input-event/            # Shared event types and scancode definitions
├── src/
│   ├── lib.rs          # Event enums (PointerEvent, KeyboardEvent, Event)
│   ├── scancode.rs     # Linux scancode mappings
│   └── error.rs        # Error types
└── Cargo.toml

lan-mouse-ipc/          # IPC library for service-frontend communication
├── src/
└── Cargo.toml

lan-mouse-cli/          # Command-line interface
├── src/
└── Cargo.toml

lan-mouse-gtk/          # GTK4/libadwaita frontend (optional feature)
├── src/
├── resources/
│   └── de.feschber.LanMouse.svg
├── build.rs            # glib-build-tools
└── Cargo.toml

lan-mouse-proto/        # Protocol definitions and event serialization
├── src/
│   └── lib.rs          # ProtoEvent, Position enums, encoding/decoding
└── Cargo.toml
```

## Essential Commands

### Building
```bash
# Standard build (includes default features)
cargo build

# Release build with optimizations
cargo build --release

# Conditional compilation (e.g., only Sway support)
cargo build --no-default-features --features layer_shell_capture,wlroots_emulation

# Specific platform builds
cargo build --features gtk  # Include GTK frontend
cargo build --features x11_capture  # Include X11 capture
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with verbose output
cargo test --verbose

# Run specific test
cargo test test_name
```

### Linting and Formatting
```bash
# Check formatting
cargo fmt --check

# Apply formatting
cargo fmt

# Run clippy (warnings cause failure in CI)
cargo clippy --all-features --all-targets -- --deny warnings

# Check for security advisories (requires cargo-deny)
cargo deny check advisories
```

### Running the Application
```bash
# Run with GTK frontend (default)
./target/debug/lan-mouse

# Run daemon mode
./target/debug/lan-mouse daemon

# Run CLI interface
./target/debug/lan-mouse cli help
```

## Code Style and Conventions

### Formatting
- **Style edition**: 2024
- **Indentation**: 4 spaces
- **Max line width**: 100 characters
- Defined in `.rustfmt.toml`

### Error Handling
- Uses `thiserror` crate for error types
- Error types derive `Debug` and `Error`
- Common pattern:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MyError {
    #[error("failed to do something: {0}")]
    SomethingFailed(String),

    #[error(transparent)]
    Io(#[from] io::Error),
}
```

### Async Patterns
- Uses `tokio` runtime with features: `io-util`, `io-std`, `macros`, `net`, `process`, `rt`, `sync`, `signal`
- Async functions use `async fn` and `.await`
- Stream processing with `futures::StreamExt`

### Configuration
- Uses `toml` for configuration files
- Config file location: `$XDG_CONFIG_HOME/lan-mouse/config.toml` (or `%LOCALAPPDATA%\lan-mouse\` on Windows)
- Configuration structs use `serde` `Serialize`/`Deserialize`
- See `config.toml` for example configuration

### Client Management
- Uses `slab::Slab` for client storage (Rc<RefCell<Slab<...>>>)
- Clients identified by `ClientHandle` (usize cast)
- ClientManager pattern in `src/client.rs`

### Key Scancode Naming
- Key symbols follow names in `input-event/src/scancode.rs`
- Example: `KeyA`, `KeyS`, `KeyLeftShift`, `KeyLeftCtrl`, `KeyLeftMeta`
- Used in `release_bind` configuration

## Architecture and Patterns

### Event Flow
```
Backend (Wayland/X11/Windows/macOS)
    ↓ InputEvent
Capture → Emitter (DTLS over UDP)
    ↓ Network
Receiver → Dispatcher
    ↓ InputEvent
Backend (Wayland/X11/Windows/macOS)
```

### Device State
- Each device is either **active** (sending events) or **inactive** (receiving events)
- Prevents feedback loops - when controlling another device, the controlled device doesn't send events back
- Managed through state machine in `ClientManager`

### Backends
**Input Capture Backends**:
- `layer_shell`: Wayland compositors with layer-shell support (Sway, Wayfire, etc.)
- `libei`: GNOME >= 45, KDE Plasma >= 6.1
- `windows`: Windows
- `macos`: macOS
- `x11`: X11 (WIP)

**Input Emulation Backends**:
- `wlroots`: wlroots compositors via virtual-pointer protocol
- `libei`: GNOME >= 45, KDE Plasma >= 6.1
- `remote_desktop_portal`: freedesktop remote desktop portal
- `windows`: Windows
- `macos`: macOS
- `x11`: X11

### Network Protocol
- UDP for event transmission (low latency)
- TCP for control requests
- DTLS encryption using `webrtc-dtls` and `rustls`
- Default port: 4242 (configurable)

## Conditional Compilation

The project uses Cargo features extensively for platform-specific code:

**Default features** (see `Cargo.toml`):
```toml
default = [
    "gtk",
    "layer_shell_capture",
    "x11_capture",
    "libei_capture",
    "wlroots_emulation",
    "libei_emulation",
    "rdp_emulation",
    "x11_emulation",
]
```

**Platform-specific dependencies**:
- Use `#[cfg(unix)]`, `#[cfg(windows)]`, `#[cfg(target_os = "macos")]` for platform gating
- Use `#[cfg(feature = "feature_name")]` for feature gating
- See `input-capture/Cargo.toml` and `input-emulation/Cargo.toml` for examples

## Testing

### Test Organization
- Unit tests in `src/` alongside implementation files
- Integration tests in `src/capture_test.rs` and `src/emulation_test.rs`
- Test commands exposed via CLI: `lan-mouse test-emulation` and `lan-mouse test-capture`

### Running Tests for Development
```bash
# Test specific functionality
cargo test --package lan-mouse

# Test specific backend
cargo test --package input-capture
```

## Important Gotchas

### 1. Backend Selection
Backends are selected automatically based on the active desktop environment. When working on a specific backend, you may need to override:
```bash
lan-mouse --capture-backend=layer-shell --emulation-backend=wlroots
```

### 2. GTK Dependencies on Windows
Building GTK from source on Windows is complex. Use the gvsbuild approach documented in README.md or disable GTK with `--no-default-features`.

### 3. libei Support
libei support requires recent GNOME (>= 45) or KDE Plasma (>= 6.1). Make sure your test environment meets requirements.

### 4. Modifier Keys on wlroots
wlroots compositors without libei support on the receiving end don't handle modifier events properly when the sending device is NOT using the layer-shell backend.

### 5. Wayfire Configuration
When using Wayfire, ensure `shortcuts-inhibit` is in the plugin list in your wayfire config, otherwise input capture won't work.

### 6. Windows Mouse Cursor
When sending input to Windows, the mouse cursor will be invisible if there's no real mouse connected to the machine.

### 7. X11 Limitations
X11 currently only has support for input emulation (can only be used as a receiver, not a sender). Input capture is WIP.

### 8. Feedback Loops
When implementing new features, always consider the device state (active/inactive) to prevent event feedback loops. The service-level logic must ensure controlled devices don't transmit events back.

### 9. Certificate Management
The application generates DTLS certificates automatically. Certificates are stored in `~/.config/lan-mouse/lan-mouse.pem` (Unix) or `%LOCALAPPDATA%\lan-mouse\lan-mouse.pem` (Windows). Fingerprints are used for authorization.

### 10. Logging
Configure logging with `LAN_MOUSE_LOG_LEVEL` environment variable:
```bash
export LAN_MOUSE_LOG_LEVEL=debug
./target/debug/lan-mouse
```

## Platform-Specific Development

### Linux
- Install dependencies: `libadwaita-1-dev`, `libgtk-4-dev`, `libx11-dev`, `libxtst-dev` (Debian/Ubuntu)
- For NixOS: use `nix-shell` or `nix develop`

### Windows
- Requires building GTK4 from source via gvsbuild
- Add `C:\gtk-build\gtk\x64\release\bin` to PATH
- Or use `--no-default-features` to disable GTK

### macOS
- Install dependencies: `brew install libadwaita pkg-config imagemagick`
- For app bundle: `cargo install cargo-bundle`, then `cargo bundle`
- Run `scripts/copy-macos-dylib.sh` to bundle dynamic libraries

## CI/CD

GitHub Actions (`.github/workflows/rust.yml`) runs:
- Build on Linux, Windows, macOS (Intel and ARM)
- Tests: `cargo test --verbose`
- Format check: `cargo fmt --check`
- Clippy: `cargo clippy --all-features --all-targets -- --deny warnings`

All checks must pass before merging.

## Adding New Features

### New Backend
1. Implement in `input-capture` or `input-emulation` crate
2. Add feature flag to respective `Cargo.toml`
3. Add platform-specific dependencies if needed
4. Update backend selection logic in `config.rs`
5. Add feature to default features in root `Cargo.toml` if appropriate

### New Event Type
1. Add to `input-event/src/lib.rs` (Event enum)
2. Update protocol in `lan-mouse-proto/src/lib.rs`
3. Implement serialization/deserialization
4. Update all backends to handle new event type

### IPC Extension
1. Add event/request type in `lan-mouse-ipc/src/lib.rs`
2. Handle in `src/service.rs` (server side) and frontend (client side)

## Key Files Reference

- `src/main.rs`: Application entry point, command routing
- `src/service.rs`: Core service implementation
- `src/config.rs`: Configuration parsing and CLI argument handling
- `src/client.rs`: Client management
- `src/capture.rs`: Input capture wrapper
- `src/emulation.rs`: Input emulation wrapper
- `src/listen.rs`: UDP listener for incoming events
- `src/connect.rs`: Connection management
- `src/crypto.rs`: DTLS certificate handling
- `src/dns.rs`: DNS resolution for hostnames
- `input-event/src/scancode.rs`: Linux scancode to key name mappings
- `lan-mouse-ipc/src/lib.rs`: IPC types and communication
- `lan-mouse-proto/src/lib.rs`: Event serialization protocol
