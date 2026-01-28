//! Screen sharing integration module
//!
//! This module handles KVM switch functionality with screen sharing,
//! allowing users to switch between remote screens using hotkeys (Ctrl+Shift+Up/Down)

use crate::client::ClientManager;
use crate::config::Config;

#[cfg(feature = "screen_share")]
use screen_share::{ScreenConfig, ScreenHotkey, ScreenMode};

#[cfg(feature = "screen_share")]
use lan_mouse_ipc::ClientHandle;
#[cfg(feature = "screen_share")]
use std::sync::Arc;
#[cfg(feature = "screen_share")]
use tokio::sync::RwLock;

/// Screen sharing manager
#[cfg(feature = "screen_share")]
pub struct ScreenShareManager {
    /// Current screen mode
    mode: Arc<RwLock<ScreenMode>>,
    /// Currently displayed client (None = local screen)
    displayed_client: Arc<RwLock<Option<ClientHandle>>>,
    /// Client manager
    client_manager: ClientManager,
    /// Screen configuration (immutable after init)
    config: ScreenConfig,
}

#[cfg(feature = "screen_share")]
impl ScreenShareManager {
    /// Create new screen sharing manager
    #[inline]
    pub fn new(config: Config, client_manager: ClientManager) -> Self {
        log::info!("Initializing screen sharing manager");

        let screen_config = ScreenConfig {
            mode: if config.enable_screen_share() {
                ScreenMode::ShareScreen
            } else {
                ScreenMode::InputOnly
            },
            fps: config.screen_share_fps(),
            quality: config.screen_share_quality(),
            hardware_accel: true,
            max_bitrate: 50,
        };

        Self {
            mode: Arc::new(RwLock::new(screen_config.mode)),
            displayed_client: Arc::new(RwLock::new(None)),
            client_manager,
            config: screen_config,
        }
    }

    /// Handle screen hotkey
    pub async fn handle_hotkey(&self, hotkey: ScreenHotkey) -> Result<(), ScreenShareError> {
        match hotkey {
            ScreenHotkey::Local => {
                log::info!("Switching to local screen");
                *self.displayed_client.write().await = None;
                self.set_mode(ScreenMode::InputOnly).await;
            }
            ScreenHotkey::Remote(client_id) => {
                log::info!("Switching to remote screen: client {}", client_id);
                *self.displayed_client.write().await = Some(client_id as ClientHandle);
                self.set_mode(ScreenMode::DisplayRemote).await;
            }
            ScreenHotkey::Cycle => {
                log::info!("Cycling through remote screens");
                self.cycle_screen().await;
            }
            ScreenHotkey::Toggle => {
                log::info!("Toggling screen sharing");
                self.toggle_screen_share().await;
            }
        }
        Ok(())
    }

    /// Cycle through available remote screens
    async fn cycle_screen(&self) {
        let current = *self.displayed_client.read().await;
        let mut displayed_client = self.displayed_client.write().await;

        // Get all active clients
        let active_clients: Vec<ClientHandle> = (0..10) // Max 10 clients
            .filter(|&h| {
                // Check if client is active
                // In real implementation, query from client_manager
                false // Placeholder
            })
            .collect();

        if active_clients.is_empty() {
            log::warn!("No active clients to cycle through");
            return;
        }

        // Find next client
        let next = match current {
            None => active_clients.first().copied(),
            Some(current_id) => {
                let pos = active_clients.iter().position(|&x| x == current_id);
                match pos {
                    Some(p) if p + 1 < active_clients.len() => active_clients.get(p + 1).copied(),
                    _ => active_clients.first().copied(),
                }
            }
        };

        *displayed_client = next;

        if next.is_some() {
            drop(displayed_client); // Release write lock before set_mode
            self.set_mode(ScreenMode::DisplayRemote).await;
        } else {
            drop(displayed_client);
            self.set_mode(ScreenMode::InputOnly).await;
        }
    }

    /// Toggle screen sharing on/off
    async fn toggle_screen_share(&self) {
        let mut mode = self.mode.write().await;
        *mode = match *mode {
            ScreenMode::InputOnly => {
                log::info!("Enabling screen sharing");
                ScreenMode::ShareScreen
            }
            ScreenMode::ShareScreen | ScreenMode::DisplayRemote => {
                log::info!("Disabling screen sharing");
                *self.displayed_client.write().await = None;
                ScreenMode::InputOnly
            }
        };
    }

    /// Set screen mode
    #[inline]
    async fn set_mode(&self, mode: ScreenMode) {
        *self.mode.write().await = mode;

        // In real implementation:
        // 1. Stop current capture/display
        // 2. Start new capture/display based on mode
        // 3. Notify remote clients of mode change

        log::info!("Screen mode changed to: {:?}", mode);
    }

    /// Get current screen mode
    #[inline]
    pub async fn current_mode(&self) -> ScreenMode {
        *self.mode.read().await
    }

    /// Get currently displayed client
    #[inline]
    pub async fn displayed_client(&self) -> Option<ClientHandle> {
        *self.displayed_client.read().await
    }

    /// Get current screen configuration
    #[inline]
    pub fn config(&self) -> &ScreenConfig {
        &self.config
    }
}

#[cfg(feature = "screen_share")]
#[derive(Debug, thiserror::Error)]
pub enum ScreenShareError {
    #[error("screen sharing not enabled")]
    NotEnabled,
    #[error("client not found: {0}")]
    ClientNotFound(ClientHandle),
    #[error("capture error: {0}")]
    Capture(String),
    #[error("display error: {0}")]
    Display(String),
}

#[cfg(not(feature = "screen_share"))]
/// No-op screen sharing manager (when feature is disabled)
pub struct ScreenShareManager;

#[cfg(not(feature = "screen_share"))]
impl ScreenShareManager {
    #[inline]
    pub fn new(_config: Config, _client_manager: ClientManager) -> Self {
        log::info!("Screen sharing disabled (feature not compiled)");
        Self
    }
}

/// Hotkey handler for Ctrl+Shift+Up/Down
#[cfg(feature = "screen_share")]
pub struct HotkeyHandler {
    manager: Arc<ScreenShareManager>,
}

#[cfg(feature = "screen_share")]
impl HotkeyHandler {
    #[inline]
    pub fn new(manager: Arc<ScreenShareManager>) -> Self {
        Self { manager }
    }

    /// Handle key combination
    #[inline]
    pub async fn handle_key(&self, ctrl: bool, shift: bool, key_code: u32) {
        if ctrl && shift {
            // Key codes for Up/Down arrows (Linux scancodes)
            // Up: 103 (Linux), 57416 (X11)
            // Down: 108 (Linux), 57424 (X11)
            match key_code {
                103 | 57416 => {
                    // Up arrow - cycle to next screen
                    if let Err(e) = self.manager.handle_hotkey(ScreenHotkey::Cycle).await {
                        log::error!("Error handling screen hotkey: {}", e);
                    }
                }
                108 | 57424 => {
                    // Down arrow - return to local screen
                    if let Err(e) = self.manager.handle_hotkey(ScreenHotkey::Local).await {
                        log::error!("Error handling screen hotkey: {}", e);
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(not(feature = "screen_share"))]
/// No-op hotkey handler
pub struct HotkeyHandler;

#[cfg(not(feature = "screen_share"))]
impl HotkeyHandler {
    #[inline]
    pub fn new(_manager: std::sync::Arc<ScreenShareManager>) -> Self {
        Self
    }

    /// No-op
    #[inline]
    pub async fn handle_key(&self, _ctrl: bool, _shift: bool, _key_code: u32) {
        // No-op
    }
}
