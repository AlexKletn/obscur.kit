//! The four GTK snapshots of a window button.

/// Native logical dimensions and PNG bytes for each interactive state.
pub struct WindowButtonImages {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) images: [std::sync::Arc<[u8]>; 4],
}

impl WindowButtonImages {
    /// Layout size in logical pixels, including the native CSS box.
    #[must_use]
    pub const fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Select one of the normal, hover, pressed and keyboard focus PNGs.
    #[must_use]
    pub fn png(&self, state: &str) -> Option<std::sync::Arc<[u8]>> {
        let index = match state {
            "normal" => 0,
            "hover" => 1,
            "pressed" => 2,
            "focus" => 3,
            _ => return None,
        };
        Some(self.images[index].clone())
    }
}
