# Screen Share Module

## Overview

The Screen Share module extends Lan Mouse from a software KVM switch (input only) to a full KVM switch with screen sharing capabilities. This allows you to use a **single monitor** to control and view multiple computers by switching between their screens.

## Key Features

- **Screen Capture**: Multi-platform screen capture using modern APIs
  - Wayland: xdg-portal or wlr-screencopy
  - Windows: Desktop Duplication API
  - macOS: ScreenCaptureKit (macOS 12.3+) or CGDisplayStream

- **Video Encoding**: AV1 codec with hardware acceleration support
  - Software encoder: rav1e (fallback)
  - Hardware acceleration: NVENC (NVIDIA), VAAPI (Intel/AMD), VideoToolbox (macOS)
  - Target: 30fps @ 50Mbps (configurable)

- **Screen Display**: Remote screen rendering
  - Wayland: EGL/GL rendering
  - Windows: Direct3D 11/12
  - macOS: Metal rendering

- **Hotkey Switching**: Quick screen switching
  - `Ctrl + Shift + Up`: Cycle to next remote screen
  - `Ctrl + Shift + Down`: Switch back to local screen

## Modes

1. **Input Only** (Default): Original Lan Mouse behavior - shares mouse/keyboard only
2. **Share Screen**: Local screen is shared to remote clients (for viewing)
3. **Display Remote**: Remote screen is displayed locally (KVM mode)

## Configuration

Add to `~/.config/lan-mouse/config.toml` (Linux/macOS) or `%LOCALAPPDATA%\lan-mouse\config.toml` (Windows):

```toml
# Enable screen sharing
enable_screen_share = true

# Frame rate (1-120 fps, default: 30)
screen_share_fps = 30

# Quality (1-100, higher = better, default: 80)
screen_share_quality = 80
```

## Command Line Options

```bash
# Enable screen sharing
lan-mouse --enable-screen-share

# Custom frame rate
lan-mouse --screen-share-fps 60

# Custom quality
lan-mouse --screen-share-quality 90
```

## How It Works

### Architecture

```
┌─────────────┐     Capture     ┌──────────────┐     Encode      ┌──────────────┐
│   Local    │ ─────────────▶ │  Screen      │ ──────────────▶ │   AV1        │
│  Computer   │               │  Capture     │               │  Encoder     │
└─────────────┘               └──────────────┘               └──────┬───────┘
                                                                        │
                                                                        ▼
                                                             ┌─────────────────┐
                                                             │  Network UDP   │
                                                             │  (WebRTC)      │
                                                             └────────┬────────┘
                                                                      │
                               ┌──────────────┐      │      ┌──────────────┐
                               │   Remote     │ ◀─────┘      │   Local      │
                               │  Computer    │               │  Monitor     │
                               └──────────────┘               └──────────────┘
                                        │
                                        ▼
                               ┌──────────────┐
                               │  Screen      │
                               │  Display     │
                               └──────────────┘
```

### Screen Switching Workflow

1. User presses `Ctrl + Shift + Up`
2. System cycles to next active remote client
3. Screen capture starts on remote computer
4. Video stream is sent via WebRTC DataChannel
5. Local computer receives and displays remote screen
6. Input events continue to be sent to remote computer

### Protocol Details

- **Transport**: WebRTC DataChannel (already used for DTLS)
- **Protocol**: JSON-based control messages + binary video data
- **Encoding**: AV1 in OBUs (Open Bitstream Units)
- **Fragmentation**: Frames split into 256KB chunks for DataChannel
- **Latency Target**: < 50ms for 30fps, < 30ms for 60fps

## Platform Support

| Platform | Capture | Display | Hardware Encode |
|----------|----------|----------|-----------------|
| Wayland (GNOME) | ✅ xdg-portal | ✅ EGL | ⚠️ VAAPI (Intel/AMD) |
| Wayland (KDE) | ✅ xdg-portal | ✅ EGL | ⚠️ VAAPI (Intel/AMD) |
| Wayland (Sway) | ✅ wlr-screencopy | ✅ EGL | ⚠️ VAAPI (Intel/AMD) |
| Windows | ✅ Desktop Duplication | ✅ D3D11/12 | ✅ NVENC (NVIDIA) |
| macOS | ✅ ScreenCaptureKit | ✅ Metal | ✅ VideoToolbox |

## Performance Optimizations

### Network
- Adaptive bitrate based on network conditions
- Frame dropping when network congested
- Prioritize keyframes (I-frames) for quick recovery

### Encoding
- Hardware acceleration when available
- Adaptive quality based on CPU/GPU load
- P-frame skipping when encoding behind

### Display
- Zero-copy rendering when possible
- GPU texture upload
- Async frame presentation

## Security

- DTLS encryption (already used in Lan Mouse)
- Certificate-based authentication
- Screen sharing requires explicit authorization
- No persistent screen data stored

## Troubleshooting

### Low Frame Rate
1. Check network bandwidth (minimum 50Mbps recommended)
2. Lower `screen_share_fps` to 15 or 20
3. Enable hardware acceleration if available

### High Latency
1. Ensure low-latency network (WiFi may have issues)
2. Reduce resolution or quality
3. Check for CPU/GPU bottlenecks

### Screen Not Displaying
1. Verify `enable_screen_share = true` in config
2. Check logs for capture/display errors
3. Ensure remote computer is authorized

## Future Enhancements

- [ ] Audio sharing alongside screen
- [ ] Multiple monitors support
- [ ] Screen recording to file
- [ ] Touch input support
- [ ] Drag and drop between computers
- [ ] Compression quality auto-tuning
- [ ] 4K resolution support
- [ ] Variable refresh rate support

## Development Notes

### Adding a New Capture Backend

1. Implement `ScreenCapture` trait in `screen-share/src/capture.rs`
2. Add platform-specific dependencies to `screen-share/Cargo.toml`
3. Add feature flag
4. Update `create_capture()` function

### Adding a New Display Backend

1. Implement `ScreenDisplay` trait in `screen-share/src/display.rs`
2. Add platform-specific dependencies
3. Add feature flag
4. Update `create_display()` function

### Testing

```bash
# Test screen capture
lan-mouse test-capture

# Test screen emulation
lan-mouse test-emulation

# With screen share enabled
lan-mouse --enable-screen-share
```

## License

GPL-3.0-or-later (same as Lan Mouse)
