//! The list a finger can drag.

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

#[doc = include_str!("../../docs/api/fixture/scrolling_list.doc/ScrollingList.md")]
#[derive(Debug)]
pub struct ScrollingList {
    /// The runtime, with one frame already rendered.
    pub runtime: UiRuntime,
    /// The scroll container node.
    pub scroller: NodeId,
    /// The six row nodes, top to bottom.
    pub rows: Vec<NodeId>,
    /// Set when any row's click handler runs.
    pub tapped: Rc<Cell<bool>>,
}

impl ScrollingList {
    /// The click command every row binds.
    pub const CMD_ROW: InternedSymbol = InternedSymbol::from_raw(0x0C);

    /// How many rows the list holds.
    pub const ROWS: u64 = 6;

    /// The height of one row, in px.
    pub const ROW_HEIGHT: f32 = 40.0;

    #[doc = include_str!("../../docs/api/fixture/scrolling_list.doc/SCROLLPORT_HEIGHT.md")]
    pub const SCROLLPORT_HEIGHT: f32 = 120.0;

    /// Renders the list with every row bound, and registers the row handler.
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

    /// The list with no event bindings.
    #[must_use]
    pub fn view() -> View {
        Self::with_rows(|_, row| row)
    }

    /// The list with every row bound to `command`.
    #[must_use]
    pub fn view_with_click(command: InternedSymbol) -> View {
        Self::with_rows(|_, row| row.with_event(ViewEventBinding::new(EVENT_CLICK, command)))
    }

    /// The sheet that sizes the scroller, makes it scrollable, and sizes the rows.
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

    /// The `(scroller, rows)` ids, after a frame has populated the tree.
    #[must_use]
    pub fn nodes(runtime: &UiRuntime) -> (NodeId, Vec<NodeId>) {
        (
            NodePath::new(&[0]).resolve(runtime),
            NodePath::new(&[0]).children(runtime),
        )
    }

    #[doc = include_str!("../../docs/api/fixture/scrolling_list.doc/with_rows.md")]
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
