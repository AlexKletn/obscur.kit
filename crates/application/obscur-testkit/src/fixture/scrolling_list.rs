#![doc = include_str!("scrolling_list.doc/_module.md")]

use std::cell::Cell;
use std::rc::Rc;

use obscur_identity::{ComponentId, InternedSymbol, NodeId};
use obscur_runtime::UiRuntime;
use obscur_style::cascade::CascadeOrigin;
use obscur_style::matching::Stylesheet;
use obscur_view::view::{
    CLASS, EVENT_CLICK, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

use crate::fixture::{ClassRule, TestFixture};
use crate::selector::NodePath;

#[doc = include_str!("scrolling_list.doc/ScrollingList.md")]
#[derive(Debug)]
pub struct ScrollingList {
    #[doc = include_str!("scrolling_list.doc/runtime.md")]
    pub runtime: UiRuntime,
    #[doc = include_str!("scrolling_list.doc/scroller.md")]
    pub scroller: NodeId,
    #[doc = include_str!("scrolling_list.doc/rows.md")]
    pub rows: Vec<NodeId>,
    #[doc = include_str!("scrolling_list.doc/tapped.md")]
    pub tapped: Rc<Cell<bool>>,
}

impl ScrollingList {
    #[doc = include_str!("scrolling_list.doc/CMD_ROW.md")]
    pub const CMD_ROW: InternedSymbol = InternedSymbol::from_raw(0x0C);

    #[doc = include_str!("scrolling_list.doc/ROWS.md")]
    pub const ROWS: u64 = 6;

    #[doc = include_str!("scrolling_list.doc/ROW_HEIGHT.md")]
    pub const ROW_HEIGHT: f32 = 40.0;

    #[doc = include_str!("scrolling_list.doc/SCROLLPORT_HEIGHT.md")]
    pub const SCROLLPORT_HEIGHT: f32 = 120.0;

    #[doc = include_str!("scrolling_list.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        let mut runtime = TestFixture::runtime();
        let tapped = Rc::new(Cell::new(false));
        {
            let t = tapped.clone();
            runtime.on_command(Self::CMD_ROW, move || t.set(true));
        }
        let sheet = Self::sheet();
        let _ = runtime.render_now(
            &Self::view_with_click(Self::CMD_ROW),
            std::slice::from_ref(&sheet),
        );
        let (scroller, rows) = Self::nodes(&runtime);
        Self {
            runtime,
            scroller,
            rows,
            tapped,
        }
    }

    #[doc = include_str!("scrolling_list.doc/view.md")]
    #[must_use]
    pub fn view() -> View {
        Self::with_rows(|_, row| row)
    }

    #[doc = include_str!("scrolling_list.doc/view_with_click.md")]
    #[must_use]
    pub fn view_with_click(command: InternedSymbol) -> View {
        Self::with_rows(|_, row| row.with_event(ViewEventBinding::new(EVENT_CLICK, command)))
    }

    #[doc = include_str!("scrolling_list.doc/sheet.md")]
    #[must_use]
    pub fn sheet() -> Stylesheet {
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                ClassRule::new("scroller")
                    .sized(200.0, Self::SCROLLPORT_HEIGHT)
                    .scrollable()
                    .build(),
                ClassRule::new("row").sized(200.0, Self::ROW_HEIGHT).build(),
            ],
        )
    }

    #[doc = include_str!("scrolling_list.doc/nodes.md")]
    #[must_use]
    pub fn nodes(runtime: &UiRuntime) -> (NodeId, Vec<NodeId>) {
        (
            NodePath::new(&[0]).resolve(runtime),
            NodePath::new(&[0]).children(runtime),
        )
    }

    #[doc = include_str!("scrolling_list.doc/with_rows.md")]
    #[must_use]
    pub fn with_rows(row: impl Fn(u64, ViewNode) -> ViewNode) -> View {
        let rows: Vec<ViewNode> = (0..Self::ROWS)
            .map(|i| {
                row(
                    i,
                    ViewNode::element(ComponentId::from_raw(200 + i))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row".into()))),
                )
            })
            .collect();
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(100))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("scroller".into())))
                .with_children(ViewFragment::from_nodes(rows)),
        ]))
    }
}

impl Default for ScrollingList {
    fn default() -> Self {
        Self::new()
    }
}
