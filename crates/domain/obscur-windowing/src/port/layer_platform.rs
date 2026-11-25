use super::{LayerCapabilities, LayerError};

/// Host port for discovering desktop panel support before opening a surface.
pub trait LayerPlatform {
    /// Reports the connected platform capabilities. Returns Unsupported when the platform cannot host a panel.
    fn capabilities(&self) -> Result<LayerCapabilities, LayerError>;
}
