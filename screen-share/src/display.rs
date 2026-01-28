use crate::VideoFrame;
use crate::error::Result;

/// Screen display backend
pub trait ScreenDisplay {
    /// Display a video frame
    fn display(&mut self, frame: &VideoFrame) -> Result<()>;

    /// Get display resolution
    fn resolution(&self) -> (u32, u32);

    /// Set display position/scale
    fn set_transform(&mut self, x: i32, y: i32, scale: f32) -> Result<()>;

    /// Start/stop display
    fn pause(&mut self) -> Result<()>;
    fn resume(&mut self) -> Result<()>;
}

/// Placeholder display backend for testing
pub struct DummyDisplay {
    paused: bool,
    resolution: (u32, u32),
}

impl DummyDisplay {
    #[inline]
    pub fn new() -> Result<Self> {
        log::info!("Using dummy screen display");
        Ok(Self {
            paused: false,
            resolution: (1920, 1080),
        })
    }
}

impl ScreenDisplay for DummyDisplay {
    fn display(&mut self, frame: &VideoFrame) -> Result<()> {
        if self.paused {
            return Ok(());
        }

        // In real implementation:
        // 1. Decode video frame if needed (AV1)
        // 2. Upload to GPU texture or software buffer
        // 3. Render to display surface

        // For now, just log (avoid log in hot path)
        #[cfg(debug_assertions)]
        log::trace!(
            "Displaying frame: {}x{} (seq: {})",
            frame.width,
            frame.height,
            frame.sequence
        );

        Ok(())
    }

    #[inline]
    fn resolution(&self) -> (u32, u32) {
        self.resolution
    }

    fn set_transform(&mut self, _x: i32, _y: i32, _scale: f32) -> Result<()> {
        // Update display transformation
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

/// Create screen display backend based on platform
#[inline]
pub fn create_display() -> Result<Box<dyn ScreenDisplay>> {
    // For now, use dummy display
    // Platform-specific implementations will be added later
    Ok(Box::new(DummyDisplay::new()?))
}
