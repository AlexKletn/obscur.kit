#![doc = include_str!("input_dispatch.doc/_module.md")]

use obscur_identity::NodeId;

use super::{KeyDispatch, PointerDispatch};

#[doc = include_str!("input_dispatch.doc/InputDispatch.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum InputDispatch {
    #[doc = include_str!("input_dispatch.doc/Pointer.md")]
    Pointer(PointerDispatch),
    #[doc = include_str!("input_dispatch.doc/Key.md")]
    Key(KeyDispatch),
    #[doc = include_str!("input_dispatch.doc/Composition.md")]
    Composition(bool),
    #[doc = include_str!("input_dispatch.doc/Scroll.md")]
    Scroll(Option<NodeId>),
}
