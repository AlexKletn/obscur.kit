#![doc = include_str!("virtual_row_list.doc/_module.md")]

use obscur_identity::ComponentId;
use obscur_view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("virtual_row_list.doc/VirtualRowList.md")]
#[derive(Debug)]
pub struct VirtualRowList;

impl VirtualRowList {
    #[doc = include_str!("virtual_row_list.doc/ROW_HEIGHT.md")]
    pub const ROW_HEIGHT: f32 = 24.0;

    #[doc = include_str!("virtual_row_list.doc/view.md")]
    #[must_use]
    pub fn view() -> View {
        let template = ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(0x00C0_0009))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row".into()))),
        ]);
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(0x00C0_0001)).with_children(
                ViewFragment::from_nodes(vec![ViewNode::virtual_collection(
                    Self::ROW_HEIGHT,
                    template,
                )]),
            ),
        ]))
    }
}
