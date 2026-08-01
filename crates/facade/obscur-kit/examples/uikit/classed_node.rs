#![doc = include_str!("classed_node.doc/_module.md")]

use obscur_kit::primitives::{Container, Text};
use obscur_kit::view::view::{CLASS, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("classed_node.doc/el.md")]
pub(super) fn el(class: &str, children: Vec<ViewNode>) -> ViewNode {
    Container::new()
        .with_children(children)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

#[doc = include_str!("classed_node.doc/text.md")]
pub(super) fn text(class: &str, content: &str) -> ViewNode {
    Text::new(content.to_owned())
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}
