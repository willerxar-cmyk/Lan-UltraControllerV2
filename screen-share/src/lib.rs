pub mod capture;
pub mod display;
pub mod encoder;
pub mod error;
pub mod protocol;

pub use error::{Result, ScreenShareError};

use std::time::Duration;

/// Screen sharing mode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ScreenMode {
    /// Only send mouse/keyboard events (default KVM behavior)
    InputOnly = 0,
    /// Share local screen to remote clients
    ShareScreen = 1,
    /// Display remote screen locally (video switching mode)
    DisplayRemote = 2,
}

impl Default for ScreenMode {
    #[inline]
    fn default() -> Self {
        Self::InputOnly
    }
}

/// Frame captured from screen
///
/// Uses `bytes::Bytes` for zero-copy operations and efficient memory management
#[derive(Clone)]
pub struct VideoFrame {
    /// Frame data (raw or encoded)
    /// Using Bytes for zero-copy and reference counting
    pub data: bytes::Bytes,
    /// Frame width in pixels
    pub width: u32,
    /// Frame height in pixels
    pub height: u32,
    /// Frame timestamp
    pub timestamp: Duration,
    /// Frame type (I/P/B frames)
    pub frame_type: FrameType,
    /// Sequence number (wrapping)
    pub sequence: u64,
}

/// Video frame type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameType {
    /// Keyframe (I-frame) - complete image
    Keyframe = 0,
    /// Inter frame (P-frame) - differences from previous frame
    Inter = 1,
    /// Bidirectional frame (B-frame) - differences from past and future
    Bidirectional = 2,
}

impl Default for FrameType {
    #[inline]
    fn default() -> Self {
        Self::Inter
    }
}

/// Screen sharing configuration
#[derive(Clone, Debug)]
pub struct ScreenConfig {
    /// Screen sharing mode
    pub mode: ScreenMode,
    /// Target frame rate (fps)
    pub fps: u32,
    /// Video quality (1-100)
    pub quality: u8,
    /// Enable hardware acceleration
    pub hardware_accel: bool,
    /// Maximum bitrate (Mbps)
    pub max_bitrate: u32,
}

impl Default for ScreenConfig {
    #[inline]
    fn default() -> Self {
        Self {
            mode: ScreenMode::default(),
            fps: 30,
            quality: 80,
            hardware_accel: true,
            max_bitrate: 50,
        }
    }
}

/// Screen hotkey for switching screens
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenHotkey {
    /// Switch to local screen
    Local,
    /// Switch to screen at position
    Remote(u32), // client id
    /// Cycle through remote screens
    Cycle,
    /// Toggle screen sharing
    Toggle,
}
