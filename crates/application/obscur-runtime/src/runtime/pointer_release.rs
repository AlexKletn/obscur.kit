//! What a pointer release resolved to: [`PointerRelease`].

use obscur_identity::NodeId;
use obscur_interaction::pointer::PointerEvent;

use super::PointerSessions;

#[doc = include_str!("../../docs/api/runtime/pointer_release.doc/PointerRelease.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PointerRelease {
    /// The node this release clicked, or [`None`] when it was not a click: no
    /// press of this pointer's own, a press on another node, a press on empty
    /// space, or an untrusted gesture.
    clicked: Option<NodeId>,
    /// The click run this release completes — `1`, `2` or `3` — carried on **this
    /// pointer's own press**, so another pointer pressing in between cannot turn
    /// a single tap into a double one.
    clicks: u32,
    /// Whether the release ended a press that had a target, i.e. whether an
    /// `:active` went out and a restyle is owed.
    press_ended: bool,
}

impl PointerRelease {
    #[doc = include_str!("../../docs/api/runtime/pointer_release.doc/take.md")]
    pub(crate) fn take(
        pointers: &mut PointerSessions,
        event: &PointerEvent,
        target: Option<NodeId>,
    ) -> Self {
        let held = pointers.take_press(event.pointer());
        let pressed = held.and_then(|press| press.target);
        Self {
            clicked: pressed
                .filter(|pressed| Some(*pressed) == target && event.trust().is_trusted()),
            clicks: held.map_or_else(|| pointers.click_count(), |press| press.clicks),
            press_ended: pressed.is_some(),
        }
    }

    /// The node this release clicked, or [`None`] when it was not a click.
    pub(crate) const fn clicked(self) -> Option<NodeId> {
        self.clicked
    }

    /// The click run this release completes (`1`, `2` or `3`).
    pub(crate) const fn clicks(self) -> u32 {
        self.clicks
    }

    /// Whether an outstanding press with a target went out with this release.
    pub(crate) const fn press_ended(self) -> bool {
        self.press_ended
    }
}

#[cfg(test)]
mod tests {
    use obscur_geometry::Point;
    use obscur_interaction::pointer::{PointerDevice, PointerEventKind, PointerId};

    use super::*;

    fn event(pointer: PointerId, kind: PointerEventKind) -> PointerEvent {
        PointerEvent::new(pointer, PointerDevice::Touch, kind, Point::new(1.0, 1.0)).as_trusted()
    }

    /// Records a press for `pointer` on `target`, the way a `Down` does.
    fn press(pointers: &mut PointerSessions, pointer: PointerId, target: NodeId, micros: u64) {
        pointers.begin_press(pointer, Some(target), Point::new(1.0, 1.0), micros);
    }

    #[test]
    fn a_release_over_the_node_its_own_press_landed_on_is_a_click() {
        let mut pointers = PointerSessions::new();
        let node = NodeId::from_raw(1);
        press(&mut pointers, PointerId::MOUSE, node, 0);
        let release = PointerRelease::take(
            &mut pointers,
            &event(PointerId::MOUSE, PointerEventKind::Up),
            Some(node),
        );
        assert_eq!(release.clicked(), Some(node));
        assert_eq!(release.clicks(), 1);
        assert!(release.press_ended());
    }

    #[test]
    fn a_second_finger_does_not_steal_the_first_fingers_release() {
        let mut pointers = PointerSessions::new();
        let (first, second) = (PointerId::from_raw(1), PointerId::from_raw(2));
        let (a, b) = (NodeId::from_raw(1), NodeId::from_raw(2));
        press(&mut pointers, first, a, 0);
        // The second contact lands mid-gesture on a different node. With one
        // anchor this used to overwrite the first finger's press.
        press(&mut pointers, second, b, 1_000);
        let release =
            PointerRelease::take(&mut pointers, &event(first, PointerEventKind::Up), Some(a));
        assert_eq!(release.clicked(), Some(a));
    }

    #[test]
    fn an_untrusted_release_clicks_nothing_but_still_ends_the_press() {
        let mut pointers = PointerSessions::new();
        let node = NodeId::from_raw(1);
        press(&mut pointers, PointerId::MOUSE, node, 0);
        // No `as_trusted`: a headless dispatch or a DevTools inject. It is still
        // observed — the press ends and `:active` goes out — but the gated default
        // action is suppressed.
        let injected = PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Touch,
            PointerEventKind::Up,
            Point::new(1.0, 1.0),
        );
        let release = PointerRelease::take(&mut pointers, &injected, Some(node));
        assert_eq!(release.clicked(), None);
        assert!(release.press_ended());
    }

    #[test]
    fn a_release_that_slid_off_its_press_is_not_a_click() {
        let mut pointers = PointerSessions::new();
        let (a, b) = (NodeId::from_raw(1), NodeId::from_raw(2));
        press(&mut pointers, PointerId::MOUSE, a, 0);
        let release = PointerRelease::take(
            &mut pointers,
            &event(PointerId::MOUSE, PointerEventKind::Up),
            Some(b),
        );
        assert_eq!(release.clicked(), None);
        // The press still went out, so the `:active` it lit has to be restyled
        // away even though nothing was activated.
        assert!(release.press_ended());
    }
}
