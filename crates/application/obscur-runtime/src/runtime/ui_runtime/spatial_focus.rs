use obscur_geometry::Point;
use obscur_interaction::command::{KeyEvent, LogicalKey, NamedKey};
use obscur_interaction::event::{EventOrigin, UiEvent};
use obscur_interaction::spatial::{NavigationDirection, SpatialNavigation};
use obscur_tree::node::NodeFlags;

use crate::frame::FrameReason;
use crate::runtime::{AncestorChain, FocusOrder, HitTesting, ScrollTargets};

use super::UiRuntime;

impl UiRuntime {
    /// Enables default directional navigation with unmodified arrows and gamepad D-pad. Disabled by default.
    pub fn set_spatial_navigation(&mut self, enabled: bool) {
        self.spatial_navigation = enabled;
    }

    /// Moves focus geometrically within the active focus trap, without wrapping. Returns whether focus changed.
    pub fn move_focus_spatially(&mut self, direction: NavigationDirection) -> bool {
        self.move_directional_focus(direction, UiEvent::detached(EventOrigin::Synthetic))
    }

    pub(super) fn try_spatial_key(&mut self, event: KeyEvent) {
        if !self.spatial_navigation || !event.modifiers().is_empty() {
            return;
        }
        let direction = match event.logical() {
            Some(LogicalKey::Named(NamedKey::ArrowUp)) => NavigationDirection::Up,
            Some(LogicalKey::Named(NamedKey::ArrowDown)) => NavigationDirection::Down,
            Some(LogicalKey::Named(NamedKey::ArrowLeft)) => NavigationDirection::Left,
            Some(LogicalKey::Named(NamedKey::ArrowRight)) => NavigationDirection::Right,
            _ => return,
        };
        self.move_directional_focus(direction, UiEvent::from_key(event));
    }

    pub(super) fn move_directional_focus(
        &mut self,
        direction: NavigationDirection,
        cause: UiEvent,
    ) -> bool {
        let order = FocusOrder::of(&self.tree, &self.computed, self.focus_trap());
        let scrolls = ScrollTargets::new(&self.tree, &self.computed, &self.boxes, &self.scroll)
            .paint(&self.computed)
            .into_iter()
            .map(|(node, paint)| (node, paint.offset()))
            .collect();
        let bounds = |node| {
            let mut bounds = self.boxes.get(&node)?.bounds;
            bounds.origin.y -=
                HitTesting::folded_point(&self.tree, &scrolls, node, Point::ORIGIN).y;
            (bounds.size.width > 0.0 && bounds.size.height > 0.0).then_some(bounds)
        };
        let candidates: Vec<_> = order
            .nodes()
            .iter()
            .copied()
            .filter_map(|node| {
                let hidden =
                    AncestorChain::of(&self.tree, Some(node))
                        .into_iter()
                        .any(|ancestor| {
                            self.tree
                                .get(ancestor)
                                .is_some_and(|element| element.has_flags(NodeFlags::HIDDEN))
                        });
                if hidden {
                    return None;
                }
                Some((node, bounds(node)?))
            })
            .collect();

        let next = if let Some(from) = self.focused.and_then(bounds) {
            SpatialNavigation::new().find_next(from, &candidates, direction)
        } else {
            candidates.first().map(|(node, _)| *node)
        };
        let Some(next) = next else {
            return false;
        };
        if !self.move_focus_to(Some(next), cause) {
            return false;
        }
        self.scroll_into_view(next);
        self.request_frame(FrameReason::Input);
        true
    }
}
