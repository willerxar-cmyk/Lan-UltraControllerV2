use thiserror::Error;

pub type Result<T> = std::result::Result<T, ScreenShareError>;

#[derive(Debug, Error)]
pub enum ScreenShareError {
    #[error("capture backend not available")]
    CaptureBackendUnavailable,

    #[error("display backend not available")]
    DisplayBackendUnavailable,

    #[error("encoder error: {0}")]
    Encoder(String),

    #[error("decoder error: {0}")]
    Decoder(String),

    #[error("frame too large: {0} bytes")]
    FrameTooLarge(usize),

    #[error("invalid frame dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },

    #[error("no screen to display")]
    NoScreenAvailable,

    #[error("hardware acceleration failed: {0}")]
    HardwareAcceleration(String),

    #[error("buffer allocation failed: {0}")]
    BufferAllocation(String),

    #[error("screen capture failed: {0}")]
    CaptureFailed(String),

    #[error("screen display failed: {0}")]
    DisplayFailed(String),

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("timeout waiting for frame")]
    Timeout,

    #[error("codec not supported: {0}")]
    CodecNotSupported(String),
}
