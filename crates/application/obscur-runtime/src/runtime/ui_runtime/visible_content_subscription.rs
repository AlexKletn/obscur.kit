use obscur_geometry::Rect;

/// Coalesces notifications for the surface’s visible content region.
pub(super) struct VisibleContentSubscription {
    pub(super) last: Option<Rect>,
    pub(super) callback: Box<dyn FnMut(Rect) -> bool>,
}

impl VisibleContentSubscription {
    pub(super) fn publish(&mut self, viewport: Rect) -> bool {
        if self.last == Some(viewport) {
            return false;
        }
        self.last = Some(viewport);
        (self.callback)(viewport)
    }
}
