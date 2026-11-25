#![doc = include_str!("../../../docs/api/runtime/ui_runtime/virtual_lists.doc/_module.md")]

use obscur_geometry::Point;
use obscur_identity::InternedSymbol;
use obscur_view::view::ViewNode;

use crate::frame::FrameReason;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/virtual_lists.doc/register_virtual_list.md")]
    pub fn register_virtual_list(
        &mut self,
        id: InternedSymbol,
        build: Box<dyn FnMut(usize) -> ViewNode>,
    ) {
        self.virtual_lists.register(id, build);
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/virtual_lists.doc/on_virtual_row_click.md")]
    pub fn on_virtual_row_click(
        &mut self,
        id: InternedSymbol,
        handler: Box<dyn FnMut(usize, u32)>,
    ) {
        self.virtual_lists.on_row_click(id, handler);
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/virtual_lists.doc/on_virtual_row_long_press.md")]
    pub fn on_virtual_row_long_press(
        &mut self,
        id: InternedSymbol,
        handler: Box<dyn FnMut(usize)>,
    ) {
        self.virtual_lists.on_row_long_press(id, handler);
    }

    /// The item range a virtual list realized on the last frame (spec §4.9).
    #[must_use]
    pub fn virtual_window(&self, id: InternedSymbol) -> Option<core::ops::Range<usize>> {
        self.virtual_lists.window(id)
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/virtual_lists.doc/virtual_index_at.md")]
    #[must_use]
    pub fn virtual_index_at(&self, screen: Point) -> Option<(InternedSymbol, usize)> {
        self.virtual_lists
            .index_at(screen, &self.boxes, &self.scroll)
    }

    /// Scrolls a registered virtual list by `dy` logical pixels, clamped to its
    /// content extent, returning the new offset (spec §4.9).
    pub fn scroll_virtual_list(&mut self, id: InternedSymbol, dy: f32, _content: f32) -> f32 {
        // Delegates to the container's own scroll, so a programmatic scroll and a
        // wheel scroll move the same thing.
        let Some(node) = self.virtual_lists.container(id) else {
            return 0.0;
        };
        let next = self.scroll_by(node, dy);
        self.request_frame(FrameReason::UiCommit);
        next
    }
}
