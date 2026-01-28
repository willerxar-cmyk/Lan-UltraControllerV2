use crate::VideoFrame;
use serde::{Deserialize, Serialize};

/// Screen sharing protocol message types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ScreenMessage {
    /// Start sharing screen
    StartShare { width: u32, height: u32, fps: u32 },
    /// Stop sharing screen
    StopShare,
    /// Request screen from remote
    RequestScreen,
    /// Video frame
    VideoFrame {
        width: u32,
        height: u32,
        timestamp_ms: u64,
        frame_type: u8, // 0=I, 1=P, 2=B
        sequence: u64,
        data_offset: u32,
        is_last: bool,
        data: Vec<u8>,
    },
    /// Frame acknowledgment
    AckFrame { sequence: u64 },
    /// Keyframe request
    RequestKeyframe,
    /// Screen mode change
    ChangeMode { mode: u8 }, // 0=InputOnly, 1=ShareScreen, 2=DisplayRemote
    /// Quality adjustment
    AdjustQuality { bitrate_mbps: u32 },
}

impl ScreenMessage {
    /// Maximum message size for WebRTC DataChannel (256KB safe limit)
    pub const MAX_SIZE: usize = 256 * 1024;

    /// Serialize message to bytes
    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        match serde_json::to_vec(self) {
            Ok(json) => {
                if json.len() > Self::MAX_SIZE {
                    return Err(format!(
                        "Message too large: {} bytes (max: {})",
                        json.len(),
                        Self::MAX_SIZE
                    ));
                }
                Ok(json)
            }
            Err(e) => Err(format!("Serialization error: {}", e)),
        }
    }

    /// Deserialize message from bytes
    pub fn deserialize(data: &[u8]) -> Result<Self, String> {
        match serde_json::from_slice(data) {
            Ok(msg) => Ok(msg),
            Err(e) => Err(format!("Deserialization error: {}", e)),
        }
    }
}

/// Protocol for streaming video frames
pub struct VideoProtocol {
    /// Current sequence number
    sequence: u64,
}

impl VideoProtocol {
    #[inline]
    pub fn new() -> Self {
        Self { sequence: 0 }
    }

    /// Split a video frame into protocol messages
    pub fn split_frame(&mut self, frame: &VideoFrame) -> Result<Vec<ScreenMessage>, String> {
        let data = frame.data.as_ref();
        // Reserve overhead for JSON headers
        let chunk_size = ScreenMessage::MAX_SIZE.saturating_sub(256);

        if data.is_empty() {
            return Ok(vec![]);
        }

        if data.len() <= chunk_size {
            // Single message
            Ok(vec![ScreenMessage::VideoFrame {
                width: frame.width,
                height: frame.height,
                timestamp_ms: frame.timestamp.as_millis() as u64,
                frame_type: match frame.frame_type {
                    crate::FrameType::Keyframe => 0,
                    crate::FrameType::Inter => 1,
                    crate::FrameType::Bidirectional => 2,
                },
                sequence: frame.sequence,
                data_offset: 0,
                is_last: true,
                data: data.to_vec(),
            }])
        } else {
            // Fragmented message
            let num_chunks = (data.len() + chunk_size - 1) / chunk_size;
            let mut messages = Vec::with_capacity(num_chunks);

            for chunk_idx in 0..num_chunks {
                let offset = chunk_idx * chunk_size;
                let end = (offset + chunk_size).min(data.len());
                let chunk = data[offset..end].to_vec();
                let is_last = end >= data.len();

                messages.push(ScreenMessage::VideoFrame {
                    width: frame.width,
                    height: frame.height,
                    timestamp_ms: frame.timestamp.as_millis() as u64,
                    frame_type: match frame.frame_type {
                        crate::FrameType::Keyframe => 0,
                        crate::FrameType::Inter => 1,
                        crate::FrameType::Bidirectional => 2,
                    },
                    sequence: frame.sequence,
                    data_offset: offset as u32,
                    is_last,
                    data: chunk,
                });
            }

            Ok(messages)
        }
    }

    /// Get next sequence number
    #[inline]
    pub fn next_sequence(&mut self) -> u64 {
        let seq = self.sequence;
        self.sequence = self.sequence.wrapping_add(1);
        seq
    }
}

impl Default for VideoProtocol {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
