use crate::VideoFrame;
use crate::error::Result;
use futures::stream::Stream;
use std::task::{Context, Poll};

/// Screen capture backend
pub trait ScreenCapture: Stream<Item = Result<VideoFrame>> + Unpin {
    /// Get current screen resolution
    fn resolution(&self) -> (u32, u32);

    /// Set capture region (None = full screen)
    fn set_region(&mut self, x: u32, y: u32, width: u32, height: u32) -> Result<()>;

    /// Start/stop capture
    fn pause(&mut self) -> Result<()>;
    fn resume(&mut self) -> Result<()>;
}

/// Placeholder capture backend for testing
pub struct DummyCapture {
    paused: bool,
    resolution: (u32, u32),
    // Pre-allocated buffer to avoid reallocation in poll_next
    _buffer: Vec<u8>,
}

impl DummyCapture {
    #[inline]
    pub fn new() -> Result<Self> {
        log::info!("Using dummy screen capture");
        let resolution = (1920, 1080);
        // Pre-allocate buffer once
        let _buffer = Vec::with_capacity(resolution.0 as usize * resolution.1 as usize * 4);

        Ok(Self {
            paused: false,
            resolution,
            _buffer,
        })
    }
}

impl Stream for DummyCapture {
    type Item = Result<VideoFrame>;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.paused {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        // In real implementation, this would poll to capture backend
        // For now, use async channel or timer-based approach
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

impl ScreenCapture for DummyCapture {
    #[inline]
    fn resolution(&self) -> (u32, u32) {
        self.resolution
    }

    fn set_region(&mut self, _x: u32, _y: u32, width: u32, height: u32) -> Result<()> {
        self.resolution = (width, height);
        // Reallocate buffer for new resolution
        // let capacity = width as usize * height as usize * 4;
        // self._buffer = Vec::with_capacity(capacity);
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        self.paused = true;
        Ok(())
    }

    fn resume(&mut self) -> Result<()> {
        self.paused = false;
        Ok(())
    }
}

/// Create screen capture backend based on platform
#[inline]
pub async fn create_capture() -> Result<Box<dyn ScreenCapture>> {
    // Platform-specific implementations will be added later
    // For now, use dummy capture
    Ok(Box::new(DummyCapture::new()?))
}
