#![doc = include_str!("../../../docs/api/runtime/ui_runtime/text_pointer.doc/_module.md")]

use obscur_geometry::Point;
use obscur_identity::NodeId;
use obscur_text::editing::EditCommand;
use obscur_text::selection::TextSelection;

use crate::orchestration::scroll_containers;
use crate::runtime::{HitTesting, ScrollTargets, TextPointerSelection, TextPress};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/text_pointer.doc/begin_text_pointer.md")]
    pub(super) fn begin_text_pointer(&mut self, target: Option<NodeId>, point: Point) {
        // A new press always ends any prior drag; it is re-armed only below.
        self.selection_scroll_point = None;
        self.pointers.set_text_drag(None);
        let clicks = self.pointers.click_count();
        let press = match self.editing.as_mut() {
            Some(session) => TextPointerSelection::new(
                &self.tree,
                &self.computed,
                &self.boxes,
                &self.scene_composer,
                session,
            )
            .press(target, point, clicks),
            None => TextPress::Ignored,
        };
        match press {
            TextPress::Ignored => {}
            // Single click: a collapsed caret at the click; `SetSelection` is
            // non-mutating, so `after_edit` (inside `apply_edit`) fires no
            // `on-input`, only a frame. Arms a potential drag-selection.
            TextPress::Caret { field, offset } => {
                self.apply_edit(EditCommand::SetSelection(TextSelection::caret(offset)));
                self.pointers.set_text_drag(Some((field, offset)));
            }
            // A word or a line: the whole range, and no drag armed.
            TextPress::Range(selection) => {
                self.apply_edit(EditCommand::SetSelection(selection));
            }
        }
    }

    /// Arms ticking when a selection pointer approaches its scrollport edge.
    pub(super) fn arm_selection_autoscroll(&mut self, point: Point) {
        let Some((field, _)) = self.pointers.text_drag() else {
            return;
        };
        let target = ScrollTargets::new(&self.tree, &self.computed, &self.boxes, &self.scroll)
            .enclosing_scrollable(field);
        self.selection_scroll_point = target
            .filter(|target| self.edge_scroll_velocity(target.node, point) != Point::ORIGIN)
            .map(|_| point);
        if self.selection_scroll_point.is_some() {
            self.request_timeline_frame();
        }
    }

    /// Scrolls under a stationary selection pointer and extends the selection at its new content position.
    pub(super) fn advance_selection_autoscroll(&mut self, delta: f32) {
        if !delta.is_finite() || delta <= 0.0 {
            return;
        }
        let Some(point) = self.selection_scroll_point.take() else {
            return;
        };
        let Some((field, _)) = self.pointers.text_drag() else {
            return;
        };
        let targets = ScrollTargets::new(&self.tree, &self.computed, &self.boxes, &self.scroll);
        let Some(container) = targets.enclosing_scrollable(field) else {
            return;
        };
        let containers = scroll_containers(&self.tree, &self.computed, &self.boxes);
        if !self.scroll_near_edge(container.node, point, delta, &containers) {
            return;
        }
        let scrolls = ScrollTargets::new(&self.tree, &self.computed, &self.boxes, &self.scroll)
            .paint(&self.computed)
            .into_iter()
            .map(|(node, paint)| (node, paint.offset()))
            .collect();
        self.update_text_drag(HitTesting::content_point(
            &self.tree, &scrolls, field, point,
        ));
        self.selection_scroll_point = Some(point);
        self.request_timeline_frame();
    }

    /// While a drag is armed, extends the selection from the press anchor to the
    /// pointer's current byte offset over the **same** field — live drag-selection
    /// (spec §5.11). A no-op when no drag is armed, no session is active, or the
    /// field's text box has gone.
    pub(super) fn update_text_drag(&mut self, point: Point) {
        let Some((field, anchor)) = self.pointers.text_drag() else {
            return;
        };
        let Some(session) = self.editing.as_mut() else {
            return;
        };
        let selection = TextPointerSelection::new(
            &self.tree,
            &self.computed,
            &self.boxes,
            &self.scene_composer,
            session,
        )
        .drag(field, anchor, point);
        if let Some(selection) = selection {
            self.apply_edit(EditCommand::SetSelection(selection));
        }
    }
}
