#![doc = include_str!("../../docs/api/runtime/pointer_event_bindings.doc/_module.md")]

use obscur_identity::InternedSymbol;
use obscur_interaction::pointer::PointerEventKind;
use obscur_tree::tree::RetainedTree;
use obscur_view::view::{
    EVENT_POINTER_CANCEL, EVENT_POINTER_DOWN, EVENT_POINTER_ENTER, EVENT_POINTER_LEAVE,
    EVENT_POINTER_MOVE, EVENT_POINTER_UP, EVENT_SCROLL_GESTURE, EVENT_TOUCH_GESTURE,
    EVENT_TOUCHPAD_GESTURE, EVENT_WHEEL,
};

/// The `pointerdown` bit.
const DOWN_BIT: u16 = 0b0000_0001;
/// The `pointerup` bit.
const UP_BIT: u16 = 0b0000_0010;
/// The `pointermove` bit.
const MOVE_BIT: u16 = 0b0000_0100;
/// The `pointercancel` bit.
const CANCEL_BIT: u16 = 0b0000_1000;
/// The `pointerenter` bit.
const ENTER_BIT: u16 = 0b0001_0000;
/// The `pointerleave` bit.
const LEAVE_BIT: u16 = 0b0010_0000;
/// The `wheel` bit.
const WHEEL_BIT: u16 = 0b0100_0000;
/// The cached presence bit for native touchpad bindings.
const TOUCHPAD_GESTURE_BIT: u16 = 0b1000_0000;
/// The cached presence bit for application touch gesture candidates.
const TOUCH_GESTURE_BIT: u16 = 1 << 8;
/// Cached presence bit for phased pixel-scroll candidates.
const SCROLL_GESTURE_BIT: u16 = 1 << 9;
#[doc = include_str!("../../docs/api/runtime/pointer_event_bindings.doc/ALL_BITS.md")]
const ALL_BITS: u16 = DOWN_BIT
    | UP_BIT
    | MOVE_BIT
    | CANCEL_BIT
    | ENTER_BIT
    | LEAVE_BIT
    | WHEEL_BIT
    | TOUCHPAD_GESTURE_BIT
    | TOUCH_GESTURE_BIT
    | SCROLL_GESTURE_BIT;

#[doc = include_str!("../../docs/api/runtime/pointer_event_bindings.doc/PointerEventBindings.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct PointerEventBindings {
    #[doc = include_str!("../../docs/api/runtime/pointer_event_bindings.doc/bound.md")]
    bound: u16,
}

impl PointerEventBindings {
    /// A tree that binds none of the seven.
    pub(crate) const fn none() -> Self {
        Self { bound: 0 }
    }

    #[doc = include_str!("../../docs/api/runtime/pointer_event_bindings.doc/name_for.md")]
    pub(crate) const fn name_for(kind: PointerEventKind) -> Option<InternedSymbol> {
        match kind {
            PointerEventKind::Down => Some(EVENT_POINTER_DOWN),
            PointerEventKind::Up => Some(EVENT_POINTER_UP),
            PointerEventKind::Move => Some(EVENT_POINTER_MOVE),
            PointerEventKind::Cancel => Some(EVENT_POINTER_CANCEL),
            // The crossing KINDS have no name of their own — deliberately, and
            // not for want of one. `pointerenter` / `pointerleave` are reserved
            // names now, but they describe a node the hover chain crossed, and
            // the chain crosses on ordinary moves far more often than the
            // pointer crosses the surface. So they are emitted from the hover
            // transition, which already sees every crossing including the two
            // these kinds cause: an `Enter` lands on a node and enters its
            // chain, a `Leave` lands on nothing and empties it. Naming them here
            // as well would dispatch each surface crossing twice.
            PointerEventKind::Enter | PointerEventKind::Leave => None,
        }
        // There is no `Wheel` kind to name here, and that is the point: a notch
        // is not a pointer report. `wheel` shares this gate — a trackpad's pixel
        // deltas arrive at nearly a move's rate — but it is dispatched from
        // `handle_scroll`, which asks for the bit by name.
    }

    #[doc = include_str!("../../docs/api/runtime/pointer_event_bindings.doc/scan.md")]
    pub(crate) fn scan(tree: &RetainedTree) -> Self {
        let mut bound = 0;
        for (_, node) in tree.iter() {
            for binding in node.event_bindings() {
                bound |= bit_for(binding.event());
            }
            // Every question this answers is already answered; the rest of the
            // tree cannot change it.
            if bound == ALL_BITS {
                break;
            }
        }
        Self { bound }
    }

    /// Whether any node binds this name. A name that is not one of the seven is never
    /// bound here, whatever else the tree carries.
    pub(crate) const fn binds(self, name: InternedSymbol) -> bool {
        let bit = bit_for(name);
        bit != 0 && self.bound & bit != 0
    }
}

/// The bit a raw pointer event name occupies, or `0` for any other symbol.
const fn bit_for(name: InternedSymbol) -> u16 {
    let raw = name.raw();
    if raw == EVENT_POINTER_DOWN.raw() {
        DOWN_BIT
    } else if raw == EVENT_POINTER_UP.raw() {
        UP_BIT
    } else if raw == EVENT_POINTER_MOVE.raw() {
        MOVE_BIT
    } else if raw == EVENT_POINTER_CANCEL.raw() {
        CANCEL_BIT
    } else if raw == EVENT_POINTER_ENTER.raw() {
        ENTER_BIT
    } else if raw == EVENT_POINTER_LEAVE.raw() {
        LEAVE_BIT
    } else if raw == EVENT_WHEEL.raw() {
        WHEEL_BIT
    } else if raw == EVENT_TOUCHPAD_GESTURE.raw() {
        TOUCHPAD_GESTURE_BIT
    } else if raw == EVENT_TOUCH_GESTURE.raw() {
        TOUCH_GESTURE_BIT
    } else if raw == EVENT_SCROLL_GESTURE.raw() {
        SCROLL_GESTURE_BIT
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use obscur_identity::ComponentId;
    use obscur_tree::node::NodeKind;
    use obscur_view::view::{EVENT_CLICK, ViewEventBinding};

    use super::*;

    fn tree_binding(events: &[InternedSymbol]) -> RetainedTree {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(1),
        });
        tree.set_root(root);
        let bindings: Vec<ViewEventBinding> = events
            .iter()
            .map(|event| ViewEventBinding::new(*event, InternedSymbol::from_raw(0x9001)))
            .collect();
        tree.get_mut(root)
            .expect("live")
            .set_event_bindings(bindings.into_boxed_slice());
        tree
    }

    #[test]
    fn an_empty_tree_binds_nothing() {
        let bindings = PointerEventBindings::scan(&RetainedTree::new());
        for name in [
            EVENT_POINTER_DOWN,
            EVENT_POINTER_UP,
            EVENT_POINTER_MOVE,
            EVENT_POINTER_CANCEL,
            EVENT_POINTER_ENTER,
            EVENT_POINTER_LEAVE,
            EVENT_WHEEL,
        ] {
            assert!(!bindings.binds(name));
        }
        assert_eq!(bindings, PointerEventBindings::none());
    }

    #[test]
    fn a_tree_that_binds_one_name_reports_only_that_one() {
        // The whole point of the gate: an application that watches moves must not
        // make presses cost anything, and the reverse.
        let bindings = PointerEventBindings::scan(&tree_binding(&[EVENT_POINTER_MOVE]));
        assert!(bindings.binds(EVENT_POINTER_MOVE));
        assert!(!bindings.binds(EVENT_POINTER_DOWN));
        assert!(!bindings.binds(EVENT_POINTER_UP));
        assert!(!bindings.binds(EVENT_POINTER_CANCEL));
        assert!(!bindings.binds(EVENT_POINTER_ENTER));
        assert!(!bindings.binds(EVENT_POINTER_LEAVE));
        assert!(!bindings.binds(EVENT_WHEEL));
    }

    #[test]
    fn a_wheel_binding_arms_only_the_wheel_bit() {
        // The wheel joined the gate rather than getting one of its own: the scan
        // already walks every live node once per structural frame, and a second
        // walk for one bit would cost what the gate exists to save. The claim it
        // has to carry is the same one — a tree that watches wheels must not make
        // a move cost anything, and the reverse.
        let bindings = PointerEventBindings::scan(&tree_binding(&[EVENT_WHEEL]));
        assert!(bindings.binds(EVENT_WHEEL));
        assert!(!bindings.binds(EVENT_POINTER_MOVE));
        assert!(!bindings.binds(EVENT_POINTER_DOWN));
        assert!(!bindings.binds(EVENT_POINTER_ENTER));
        // And the reverse: a tree that watches moves does not arm the wheel, so
        // `handle_scroll` stays free for the overwhelmingly common application
        // that binds neither.
        let moves = PointerEventBindings::scan(&tree_binding(&[EVENT_POINTER_MOVE]));
        assert!(!moves.binds(EVENT_WHEEL));
    }

    #[test]
    fn a_crossing_binding_arms_only_its_own_bit() {
        // The crossings share the gate with the four raw kinds, so the same claim
        // has to hold across the two halves: a tree that watches enters must not
        // make a leave — or a move — cost anything.
        let bindings = PointerEventBindings::scan(&tree_binding(&[EVENT_POINTER_ENTER]));
        assert!(bindings.binds(EVENT_POINTER_ENTER));
        assert!(!bindings.binds(EVENT_POINTER_LEAVE));
        assert!(!bindings.binds(EVENT_POINTER_MOVE));
    }

    #[test]
    fn an_unrelated_binding_does_not_arm_the_gate() {
        // An `on:click` is the overwhelmingly common case and it must leave the
        // raw path completely cold.
        let bindings = PointerEventBindings::scan(&tree_binding(&[EVENT_CLICK]));
        assert_eq!(bindings, PointerEventBindings::none());
        assert!(!bindings.binds(EVENT_CLICK), "not a raw pointer name");
    }

    #[test]
    fn every_dispatchable_kind_has_a_name_and_the_crossing_kinds_route_elsewhere() {
        assert_eq!(
            PointerEventBindings::name_for(PointerEventKind::Down),
            Some(EVENT_POINTER_DOWN)
        );
        assert_eq!(
            PointerEventBindings::name_for(PointerEventKind::Up),
            Some(EVENT_POINTER_UP)
        );
        assert_eq!(
            PointerEventBindings::name_for(PointerEventKind::Move),
            Some(EVENT_POINTER_MOVE)
        );
        assert_eq!(
            PointerEventBindings::name_for(PointerEventKind::Cancel),
            Some(EVENT_POINTER_CANCEL)
        );
        // 2026-08-03. This used to assert that the crossing kinds have no name
        // at all, because `pointerenter` / `pointerleave` did not exist. They do
        // now, and the rule that replaced it is narrower and still one-way: a
        // crossing KIND is not dispatched from here. The crossing NAMES are
        // emitted from the hover transition, which sees these two kinds' own
        // crossings along with every other, so naming them here as well would
        // dispatch each surface crossing twice.
        assert_eq!(
            PointerEventBindings::name_for(PointerEventKind::Enter),
            None
        );
        assert_eq!(
            PointerEventBindings::name_for(PointerEventKind::Leave),
            None
        );
        // And the other direction, so "routes elsewhere" is not "is unreachable":
        // the two names exist and the gate knows them.
        assert_ne!(bit_for(EVENT_POINTER_ENTER), 0);
        assert_ne!(bit_for(EVENT_POINTER_LEAVE), 0);
        // The wheel is the same shape of claim from the other side: it has no
        // kind at all — there is no `PointerEventKind::Wheel` to pass in — and
        // the gate still knows its name, because `handle_scroll` asks for the bit
        // by name rather than by kind.
        assert_ne!(bit_for(EVENT_WHEEL), 0);
    }

    #[test]
    fn every_gated_name_has_its_own_bit_and_the_scan_knows_the_complete_set() {
        let mut handed_out = 0;
        for name in [
            EVENT_POINTER_DOWN,
            EVENT_POINTER_UP,
            EVENT_POINTER_MOVE,
            EVENT_POINTER_CANCEL,
            EVENT_POINTER_ENTER,
            EVENT_POINTER_LEAVE,
            EVENT_WHEEL,
            EVENT_TOUCHPAD_GESTURE,
            EVENT_TOUCH_GESTURE,
            EVENT_SCROLL_GESTURE,
        ] {
            let bit = bit_for(name);
            assert!(bit.is_power_of_two());
            assert_eq!(handed_out & bit, 0, "bindings must have independent bits");
            handed_out |= bit;
        }
        assert_eq!(handed_out, ALL_BITS);
        assert!(ALL_BITS.count_ones() < u16::BITS);
    }
}
