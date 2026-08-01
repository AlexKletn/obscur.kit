#![doc = include_str!("virtual_lists.doc/_module.md")]

use std::cell::RefCell;
use std::collections::HashMap;

use obscur_identity::{InternedSymbol, NodeId};
use obscur_layout::model::Dimension;
use obscur_layout::virtualization::{OverscanPolicy, SizeEstimator, VirtualLayout};
use obscur_tree::tree::RetainedTree;
use obscur_view::view::{
    VIRTUAL_SPACER, View, ViewNode, realize_virtual_collections, realize_virtual_lists,
    virtual_scroll_resets,
};

use super::ScrollOffsets;
use crate::orchestration::{LayoutBoxes, LayoutStyles};
use obscur_geometry::Point;

#[doc = include_str!("virtual_lists.doc/VIRTUAL_OVERSCAN.md")]
const VIRTUAL_OVERSCAN: usize = 3;

#[doc = include_str!("virtual_lists.doc/ASSUMED_VIRTUAL_ITEM_COUNT.md")]
const ASSUMED_VIRTUAL_ITEM_COUNT: usize = 1_000_000;

#[doc = include_str!("virtual_lists.doc/VirtualLists.md")]
pub(crate) struct VirtualLists {
    #[doc = include_str!("virtual_lists.doc/builders.md")]
    builders: HashMap<InternedSymbol, Box<dyn FnMut(usize) -> ViewNode>>,
    #[doc = include_str!("virtual_lists.doc/containers.md")]
    containers: HashMap<InternedSymbol, NodeId>,
    #[doc = include_str!("virtual_lists.doc/windows.md")]
    windows: HashMap<InternedSymbol, core::ops::Range<usize>>,
    #[doc = include_str!("virtual_lists.doc/row_extents.md")]
    row_extents: HashMap<InternedSymbol, f32>,
    #[doc = include_str!("virtual_lists.doc/scroll_reset_seen.md")]
    scroll_reset_seen: HashMap<InternedSymbol, u64>,
    #[doc = include_str!("virtual_lists.doc/row_click_handlers.md")]
    row_click_handlers: HashMap<InternedSymbol, Box<dyn FnMut(usize, u32)>>,
    #[doc = include_str!("virtual_lists.doc/row_hold_handlers.md")]
    row_hold_handlers: HashMap<InternedSymbol, Box<dyn FnMut(usize)>>,
}

impl VirtualLists {
    #[doc = include_str!("virtual_lists.doc/new.md")]
    pub(crate) fn new() -> Self {
        Self {
            builders: HashMap::new(),
            containers: HashMap::new(),
            windows: HashMap::new(),
            row_extents: HashMap::new(),
            scroll_reset_seen: HashMap::new(),
            row_click_handlers: HashMap::new(),
            row_hold_handlers: HashMap::new(),
        }
    }

    #[doc = include_str!("virtual_lists.doc/register.md")]
    pub(crate) fn register(
        &mut self,
        id: InternedSymbol,
        build: Box<dyn FnMut(usize) -> ViewNode>,
    ) {
        self.builders.insert(id, build);
    }

    #[doc = include_str!("virtual_lists.doc/on_row_click.md")]
    pub(crate) fn on_row_click(&mut self, id: InternedSymbol, handler: Box<dyn FnMut(usize, u32)>) {
        self.row_click_handlers.insert(id, handler);
    }

    #[doc = include_str!("virtual_lists.doc/on_row_long_press.md")]
    pub(crate) fn on_row_long_press(&mut self, id: InternedSymbol, handler: Box<dyn FnMut(usize)>) {
        self.row_hold_handlers.insert(id, handler);
    }

    #[doc = include_str!("virtual_lists.doc/container.md")]
    pub(crate) fn container(&self, id: InternedSymbol) -> Option<NodeId> {
        self.containers.get(&id).copied()
    }

    #[doc = include_str!("virtual_lists.doc/window.md")]
    pub(crate) fn window(&self, id: InternedSymbol) -> Option<core::ops::Range<usize>> {
        self.windows.get(&id).cloned()
    }

    #[doc = include_str!("virtual_lists.doc/fire_row_click.md")]
    pub(crate) fn fire_row_click(&mut self, id: InternedSymbol, index: usize, clicks: u32) -> bool {
        if let Some(handler) = self.row_click_handlers.get_mut(&id) {
            handler(index, clicks);
            return true;
        }
        false
    }

    #[doc = include_str!("virtual_lists.doc/fire_row_hold.md")]
    pub(crate) fn fire_row_hold(&mut self, id: InternedSymbol, index: usize) {
        if let Some(handler) = self.row_hold_handlers.get_mut(&id) {
            handler(index);
        }
    }

    #[doc = include_str!("virtual_lists.doc/index_at.md")]
    pub(crate) fn index_at(
        &self,
        screen: Point,
        boxes: &LayoutBoxes,
        scroll: &ScrollOffsets,
    ) -> Option<(InternedSymbol, usize)> {
        for (&id, &container) in &self.containers {
            let Some(layout_box) = boxes.get(&container) else {
                continue;
            };
            // The click must be within the visible scrollport, tested on screen — a
            // scrolled container clips its content on screen, so a point below the box
            // is not a row of this list even though its content extends further.
            if !layout_box.bounds.contains(screen) {
                continue;
            }
            let Some(&extent) = self.row_extents.get(&id) else {
                continue;
            };
            if extent <= 0.0 {
                continue;
            }
            // Screen y relative to the container top, plus the scroll the content is
            // shifted by, is the offset into the full list; divide by the row extent.
            let offset = (screen.y - layout_box.bounds.origin.y) + scroll.stored(container);
            if offset < 0.0 {
                continue;
            }
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "offset is non-negative and bounded by the container's clamped scrollport"
            )]
            let index = (offset / extent) as usize;
            return Some((id, index));
        }
        None
    }

    #[doc = include_str!("virtual_lists.doc/holdable_row_at.md")]
    pub(crate) fn holdable_row_at(
        &self,
        screen: Point,
        boxes: &LayoutBoxes,
        scroll: &ScrollOffsets,
    ) -> Option<(InternedSymbol, usize)> {
        self.index_at(screen, boxes, scroll)
            .filter(|(id, _)| self.row_hold_handlers.contains_key(id))
    }

    #[doc = include_str!("virtual_lists.doc/track_containers.md")]
    pub(crate) fn track_containers(&mut self, tree: &RetainedTree) {
        for (_, node) in tree.iter() {
            if node.kind().component_id() != Some(VIRTUAL_SPACER) {
                continue;
            }
            let Some(list) = node.attributes().virtual_list() else {
                continue;
            };
            let Some(parent) = node.logical_parent().parent() else {
                continue;
            };
            self.containers
                .insert(InternedSymbol::from_raw(list), parent);
        }
    }

    #[doc = include_str!("virtual_lists.doc/realize.md")]
    pub(crate) fn realize(
        &mut self,
        view: &View,
        viewport_height: f32,
        viewport_scroll_y: f32,
        scroll: &mut ScrollOffsets,
    ) -> View {
        // `insert` returns the token last seen, so a differing or absent one
        // triggers the reset.
        for (id, token) in virtual_scroll_resets(view) {
            if self.scroll_reset_seen.insert(id, token) != Some(token)
                && let Some(&container) = self.containers.get(&id)
            {
                scroll.seat(container, 0.0);
            }
        }
        // A registered virtual list realizes through its builder: the runtime asks
        // `VirtualLayout` for the window against THIS list's own scroll offset,
        // builds only those rows, and brackets them with spacers so the container's
        // extent stays the full `count * extent`. That is the half an app cannot
        // state for itself.
        let mut builders = std::mem::take(&mut self.builders);
        let scrolls: HashMap<InternedSymbol, f32> = self
            .containers
            .iter()
            .map(|(&id, &node)| (id, scroll.stored(node)))
            .collect();
        let mut chosen: HashMap<InternedSymbol, core::ops::Range<usize>> = HashMap::new();
        // The row extent each list realizes against, captured here so a later click
        // on a row can be resolved to a model index.
        let mut extents: HashMap<InternedSymbol, f32> = HashMap::new();
        let with_lists = if builders.is_empty() {
            view.clone()
        } else {
            let picked = RefCell::new(&mut chosen);
            let sized = RefCell::new(&mut extents);
            realize_virtual_lists(
                view.clone(),
                &|id, count, extent| {
                    let offset = scrolls.get(&id).copied().unwrap_or(0.0);
                    let layout = VirtualLayout::new(
                        SizeEstimator::new(extent),
                        OverscanPolicy::symmetric(VIRTUAL_OVERSCAN),
                    );
                    let range = layout.visible_range(offset, viewport_height, count);
                    let range = range.start()..range.end();
                    picked.borrow_mut().insert(id, range.clone());
                    sized.borrow_mut().insert(id, extent);
                    range
                },
                &mut |id, index| builders.get_mut(&id).map(|build| build(index)),
            )
        };
        self.builders = builders;
        self.windows.extend(chosen);
        self.row_extents.extend(extents);
        realize_virtual_collections(with_lists, &|extent| {
            Self::template_window_len(extent, viewport_height, viewport_scroll_y)
        })
    }

    #[doc = include_str!("virtual_lists.doc/size_spacers.md")]
    pub(crate) fn size_spacers(tree: &RetainedTree, layout_styles: &mut LayoutStyles) {
        let spacers: Vec<(NodeId, f32)> = tree
            .iter()
            .filter_map(|(id, node)| {
                (node.kind().component_id() == Some(VIRTUAL_SPACER))
                    .then(|| node.attributes().virtual_extent().map(|h| (id, h)))
                    .flatten()
            })
            .collect();
        for (node, height) in spacers {
            if let Some(style) = layout_styles.get_mut(&node) {
                style.height = Dimension::Px(height);
            }
        }
    }

    #[doc = include_str!("virtual_lists.doc/dump_geometry.md")]
    pub(crate) fn dump_geometry(
        &self,
        tree: &RetainedTree,
        boxes: &LayoutBoxes,
        scroll: &ScrollOffsets,
    ) {
        if std::env::var_os("OBSCUR_VLIST_DEBUG").is_none() {
            return;
        }
        for (&id, &container) in &self.containers {
            let Some(node) = tree.get(container) else {
                continue;
            };
            let offset = scroll.stored(container);
            let window = self.windows.get(&id);
            println!("[vlist {id:?}] scroll {offset:.1} window {window:?}");
            for child in node.children() {
                let kind = tree
                    .get(*child)
                    .and_then(|n| n.kind().component_id())
                    .map_or_else(|| "?".to_owned(), |c| format!("{:#x}", c.raw()));
                if let Some(b) = boxes.get(child) {
                    // Every text this row carries. Layout showing one clean box while
                    // the screen shows two names means the row holds more text than
                    // it should — a stale run left beside the new one.
                    let mut texts = Vec::new();
                    let mut stack = vec![*child];
                    while let Some(id) = stack.pop() {
                        let Some(n) = tree.get(id) else { continue };
                        if let Some(t) = n.kind().text() {
                            texts.push(t.to_string());
                        }
                        stack.extend(n.child_nodes().iter().copied());
                    }
                    println!(
                        "    {kind:>10}  y {:>9.1}  h {:>8.1}  texts {texts:?}",
                        b.origin().y,
                        b.size().height
                    );
                }
            }
        }
    }

    #[doc = include_str!("virtual_lists.doc/template_window_len.md")]
    fn template_window_len(extent: f32, viewport_height: f32, scroll_y: f32) -> usize {
        // A non-positive estimate would make the size walk never advance; fall
        // back to the shared default rather than loop.
        let row_extent = if extent > 0.0 {
            extent
        } else {
            obscur_view::view::DEFAULT_VIRTUAL_ROW_EXTENT
        };
        let layout = VirtualLayout::new(
            SizeEstimator::new(row_extent),
            OverscanPolicy::symmetric(VIRTUAL_OVERSCAN),
        );
        layout
            .visible_range(scroll_y, viewport_height, ASSUMED_VIRTUAL_ITEM_COUNT)
            .len()
    }
}

#[cfg(test)]
mod tests {
    use super::{ScrollOffsets, VirtualLists};
    use crate::orchestration::{LayoutBoxes, LayoutStyles};
    use obscur_geometry::{Point, Rect, Size};
    use obscur_identity::{ComponentId, InternedSymbol, NodeId};
    use obscur_layout::model::{Dimension, LayoutBox, LayoutStyle};
    use obscur_tree::node::{LogicalParent, NodeKind};
    use obscur_tree::tree::RetainedTree;
    use obscur_view::view::{
        VIRTUAL_LIST_ID, VIRTUAL_ROW_EXTENT, VIRTUAL_SPACER, View, ViewFragment, ViewNode,
        ViewProperty, ViewValue,
    };
    use std::cell::RefCell;
    use std::rc::Rc;

    const LIST: InternedSymbol = InternedSymbol::from_raw(0x0000_0777);
    /// The rows are 20px tall and the viewport shows 100px of them, so a window is
    /// a handful of rows however long the model is.
    const ROW: f32 = 20.0;
    const VIEWPORT: f32 = 100.0;
    const ITEMS: usize = 1_000;

    /// A tree shaped the way reconciliation leaves a realized list — `root →
    /// container → spacer` — with the spacer naming `LIST` and standing in for
    /// 400px of unbuilt rows. Returns the tree and the container's id.
    fn realized_tree() -> (RetainedTree, NodeId, NodeId) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let container = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(1),
        });
        tree.get_mut(container)
            .expect("just allocated")
            .set_logical_parent(LogicalParent::of(root));
        tree.get_mut(root).expect("in tree").push_child(container);
        let spacer = tree.alloc(NodeKind::Element {
            component: VIRTUAL_SPACER,
        });
        {
            let node = tree.get_mut(spacer).expect("just allocated");
            node.set_logical_parent(LogicalParent::of(container));
            node.set_attributes(obscur_view::view::StyleAttributes::extract(&[
                ViewProperty::new(VIRTUAL_LIST_ID, ViewValue::Int(i64::from(LIST.raw()))),
                ViewProperty::new(VIRTUAL_ROW_EXTENT, ViewValue::Float(400.0)),
            ]));
        }
        tree.get_mut(container).expect("in tree").push_child(spacer);
        (tree, container, spacer)
    }

    /// A view holding one virtual-list marker for `LIST`, optionally tagged with a
    /// scroll-reset `token`.
    fn marker(token: Option<u64>) -> View {
        let mut node = ViewNode::virtual_list(LIST, ITEMS, ROW);
        if let Some(token) = token {
            node = node.with_virtual_scroll_reset(token);
        }
        View::from_fragment(ViewFragment::from_nodes(vec![node]))
    }

    /// A list registered with a trivial row builder, already tracked to
    /// `container`.
    fn lists(tree: &RetainedTree) -> VirtualLists {
        let mut lists = VirtualLists::new();
        lists.register(
            LIST,
            Box::new(|index| ViewNode::text(format!("row {index}"))),
        );
        lists.track_containers(tree);
        lists
    }

    /// The container's on-screen box: 100px of scrollport at y = 100.
    fn boxes_at(container: NodeId) -> LayoutBoxes {
        let mut boxes = LayoutBoxes::new();
        boxes.insert(
            container,
            LayoutBox::new(Rect::new(
                Point::new(0.0, 100.0),
                Size::new(100.0, VIEWPORT),
            )),
        );
        boxes
    }

    #[test]
    fn a_window_and_the_extent_it_realized_at_are_recorded_together() {
        // The invariant, asserted from the outside: after one `realize` the list has
        // a window *and* a point in its container resolves to a row. A window
        // recorded without its extent would leave `index_at` with nothing to divide
        // by, and every row of the list would silently stop responding.
        let (tree, container, _) = realized_tree();
        let mut lists = lists(&tree);
        let mut scroll = ScrollOffsets::new();
        lists.realize(&marker(None), VIEWPORT, 0.0, &mut scroll);

        assert!(lists.window(LIST).is_some(), "the window was recorded");
        assert_eq!(
            lists.index_at(Point::new(10.0, 150.0), &boxes_at(container), &scroll),
            Some((LIST, 2)),
            "…and so was the extent that makes it point-resolvable"
        );
    }

    #[test]
    fn a_replaced_model_resets_the_scroll_before_the_window_is_computed() {
        // The ordering the two halves of `realize` exist to fuse: the app replaced
        // the list's content, so the offset belongs to the old content and the
        // window must be computed from zero — not from where the *previous* content
        // was scrolled to.
        let (tree, container, _) = realized_tree();
        let mut lists = lists(&tree);
        let mut scroll = ScrollOffsets::new();

        // Scrolled deep into the old content…
        scroll.seat(container, 5_000.0);
        lists.realize(&marker(Some(1)), VIEWPORT, 0.0, &mut scroll);
        assert_eq!(
            scroll.stored(container),
            0.0,
            "a changed token seats the container at the top"
        );
        assert_eq!(
            lists.window(LIST).expect("a window").start,
            0,
            "and the window is computed from the offset the reset just wrote"
        );
    }

    #[test]
    fn an_unchanged_token_leaves_the_scroll_where_the_user_put_it() {
        // The other side of the same branch: re-presenting the *same* content keeps
        // the position, so a list that merely grew in place does not jump to the top.
        let (tree, container, _) = realized_tree();
        let mut lists = lists(&tree);
        let mut scroll = ScrollOffsets::new();
        lists.realize(&marker(Some(7)), VIEWPORT, 0.0, &mut scroll);

        scroll.seat(container, 5_000.0);
        lists.realize(&marker(Some(7)), VIEWPORT, 0.0, &mut scroll);
        assert_eq!(scroll.stored(container), 5_000.0);
        assert!(
            lists.window(LIST).expect("a window").start > 200,
            "5000px of 20px rows is row 250, so the window follows the scroll"
        );
    }

    #[test]
    fn a_list_with_no_builder_records_no_window() {
        // A marker naming a list nobody registered realizes no rows, so there is no
        // window and no extent to claim otherwise.
        let (tree, _, _) = realized_tree();
        let mut lists = VirtualLists::new();
        lists.track_containers(&tree);
        let mut scroll = ScrollOffsets::new();
        lists.realize(&marker(None), VIEWPORT, 0.0, &mut scroll);
        assert!(lists.window(LIST).is_none());
    }

    #[test]
    fn a_row_index_is_the_scrolled_offset_over_the_extent() {
        // The click-to-model-index arithmetic, which is only correct if the extent
        // and the container it is keyed beside describe the same realization.
        let (tree, container, _) = realized_tree();
        let mut lists = lists(&tree);
        let mut scroll = ScrollOffsets::new();
        lists.realize(&marker(None), VIEWPORT, 0.0, &mut scroll);
        let boxes = boxes_at(container);

        // Unscrolled: 10px into a 100px-high container of 20px rows is row 0.
        assert_eq!(
            lists.index_at(Point::new(5.0, 110.0), &boxes, &scroll),
            Some((LIST, 0))
        );
        // Scrolled by 60px: the same pixel is now (10 + 60) / 20 = row 3.
        scroll.seat(container, 60.0);
        assert_eq!(
            lists.index_at(Point::new(5.0, 110.0), &boxes, &scroll),
            Some((LIST, 3))
        );
    }

    #[test]
    fn a_point_outside_the_container_is_not_a_row() {
        // A scrolled container clips on screen, so a point below its box belongs to
        // whatever is painted there — not to the rows its content still extends over.
        let (tree, container, _) = realized_tree();
        let mut lists = lists(&tree);
        let mut scroll = ScrollOffsets::new();
        lists.realize(&marker(None), VIEWPORT, 0.0, &mut scroll);
        let boxes = boxes_at(container);
        assert_eq!(lists.index_at(Point::new(5.0, 99.0), &boxes, &scroll), None);
        assert_eq!(
            lists.index_at(Point::new(5.0, 250.0), &boxes, &scroll),
            None
        );
    }

    #[test]
    fn a_hold_is_offered_only_where_the_list_registered_one() {
        // Arming a hold on every contact would spend half a second of frames on
        // every tap, so the recognizer asks first — and for a virtual row the answer
        // needs the geometry and the handler map at once.
        let (tree, container, _) = realized_tree();
        let mut lists = lists(&tree);
        let mut scroll = ScrollOffsets::new();
        lists.realize(&marker(None), VIEWPORT, 0.0, &mut scroll);
        let boxes = boxes_at(container);
        let point = Point::new(5.0, 150.0);

        assert!(lists.index_at(point, &boxes, &scroll).is_some());
        assert_eq!(
            lists.holdable_row_at(point, &boxes, &scroll),
            None,
            "no long-press handler: the row is there but nothing is bound to it"
        );
        lists.on_row_long_press(LIST, Box::new(|_| {}));
        assert_eq!(
            lists.holdable_row_at(point, &boxes, &scroll),
            Some((LIST, 2))
        );
    }

    #[test]
    fn a_fired_row_click_reports_whether_a_handler_ran() {
        // The runtime schedules a frame off this answer, so an unregistered list must
        // say `false` rather than quietly costing a frame per click.
        let mut lists = VirtualLists::new();
        assert!(!lists.fire_row_click(LIST, 4, 1));

        let seen = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&seen);
        lists.on_row_click(
            LIST,
            Box::new(move |index, clicks| sink.borrow_mut().push((index, clicks))),
        );
        assert!(lists.fire_row_click(LIST, 4, 2));
        assert_eq!(*seen.borrow(), vec![(4, 2)]);
    }

    #[test]
    fn a_fired_row_hold_reaches_only_its_own_list() {
        let mut lists = VirtualLists::new();
        let seen = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&seen);
        lists.on_row_long_press(LIST, Box::new(move |index| sink.borrow_mut().push(index)));
        lists.fire_row_hold(LIST, 9);
        lists.fire_row_hold(InternedSymbol::from_raw(0x0000_0778), 3);
        assert_eq!(*seen.borrow(), vec![9]);
    }

    #[test]
    fn a_spacer_is_sized_to_the_rows_it_stands_in_for() {
        // Without this the container measures only the window, and the scrollbar
        // claims a million-row list is a screenful long.
        let (tree, container, spacer) = realized_tree();
        let mut styles = LayoutStyles::new();
        styles.insert(container, LayoutStyle::default());
        styles.insert(spacer, LayoutStyle::default());
        VirtualLists::size_spacers(&tree, &mut styles);
        assert_eq!(styles[&spacer].height, Dimension::Px(400.0));
        assert_eq!(
            styles[&container].height,
            LayoutStyle::default().height,
            "only spacers are patched"
        );
    }

    #[test]
    fn a_container_is_found_from_the_spacer_that_names_its_list() {
        // The tracking that makes the window and the visible scroll the same thing:
        // the spacer's *parent* is the box the user scrolls.
        let (tree, container, _) = realized_tree();
        let mut lists = VirtualLists::new();
        assert_eq!(lists.container(LIST), None);
        lists.track_containers(&tree);
        assert_eq!(lists.container(LIST), Some(container));
    }
}
