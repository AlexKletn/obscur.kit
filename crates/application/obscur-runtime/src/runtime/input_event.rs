#![doc = include_str!("input_event.doc/_module.md")]

use obscur_geometry::Point;
use obscur_interaction::command::KeyEvent;
use obscur_interaction::pointer::PointerEvent;
use obscur_text::ime::CompositionEvent;

#[doc = include_str!("input_event.doc/InputEvent.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    #[doc = include_str!("input_event.doc/Pointer.md")]
    Pointer(PointerEvent),
    #[doc = include_str!("input_event.doc/Key.md")]
    Key(KeyEvent),
    #[doc = include_str!("input_event.doc/Composition.md")]
    Composition(CompositionEvent),
    #[doc = include_str!("input_event.doc/Scroll.md")]
    Scroll {
        #[doc = include_str!("input_event.doc/delta.md")]
        delta: f32,
        #[doc = include_str!("input_event.doc/position.md")]
        position: Point,
    },
}
