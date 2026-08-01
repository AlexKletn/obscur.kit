#![doc = include_str!("pointer_release.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_interaction::pointer::PointerEvent;

use super::PointerSessions;

#[doc = include_str!("pointer_release.doc/PointerRelease.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PointerRelease {
    #[doc = include_str!("pointer_release.doc/clicked.md")]
    clicked: Option<NodeId>,
    #[doc = include_str!("pointer_release.doc/clicks.md")]
    clicks: u32,
    #[doc = include_str!("pointer_release.doc/press_ended.md")]
    press_ended: bool,
}

impl PointerRelease {
    #[doc = include_str!("pointer_release.doc/take.md")]
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

    #[doc = include_str!("pointer_release.doc/clicked_2.md")]
    pub(crate) const fn clicked(self) -> Option<NodeId> {
        self.clicked
    }

    #[doc = include_str!("pointer_release.doc/clicks_2.md")]
    pub(crate) const fn clicks(self) -> u32 {
        self.clicks
    }

    #[doc = include_str!("pointer_release.doc/press_ended_2.md")]
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
