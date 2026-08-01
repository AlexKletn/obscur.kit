#![doc = include_str!("virtual_collection.doc/_module.md")]

use obscur_identity::{ComponentId, InternedSymbol};

use super::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("virtual_collection.doc/VIRTUAL_COLLECTION.md")]
pub const VIRTUAL_COLLECTION: ComponentId = ComponentId::from_raw(0x00B0_0002);

#[doc = include_str!("virtual_collection.doc/VIRTUAL_ROW_EXTENT.md")]
pub const VIRTUAL_ROW_EXTENT: InternedSymbol = InternedSymbol::from_raw(0x0000_0302);

#[doc = include_str!("virtual_collection.doc/VIRTUAL_SPACER.md")]
pub const VIRTUAL_SPACER: ComponentId = ComponentId::from_raw(0x00B0_0003);

#[doc = include_str!("virtual_collection.doc/VIRTUAL_LIST_ID.md")]
pub const VIRTUAL_LIST_ID: InternedSymbol = InternedSymbol::from_raw(0x0000_0303);

#[doc = include_str!("virtual_collection.doc/VIRTUAL_ITEM_COUNT.md")]
pub const VIRTUAL_ITEM_COUNT: InternedSymbol = InternedSymbol::from_raw(0x0000_0304);

#[doc = include_str!("virtual_collection.doc/VIRTUAL_SCROLL_RESET.md")]
pub const VIRTUAL_SCROLL_RESET: InternedSymbol = InternedSymbol::from_raw(0x0000_0305);

#[doc = include_str!("virtual_collection.doc/DEFAULT_VIRTUAL_ROW_EXTENT.md")]
pub const DEFAULT_VIRTUAL_ROW_EXTENT: f32 = 24.0;

#[doc = include_str!("virtual_collection.doc/virtual_row_extent.md")]
#[must_use]
pub fn virtual_row_extent(props: &[ViewProperty]) -> Option<f32> {
    props
        .iter()
        .find(|prop| prop.name() == VIRTUAL_ROW_EXTENT)
        .and_then(|prop| match prop.value() {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "the extent is authored as a small pixel count; f32 is its real precision"
            )]
            ViewValue::Float(extent) => Some(*extent as f32),
            ViewValue::Int(extent) => Some(*extent as f32),
            _ => None,
        })
}

#[doc = include_str!("virtual_collection.doc/is_virtual_collection.md")]
#[must_use]
pub fn is_virtual_collection(node: &ViewNode) -> bool {
    matches!(node, ViewNode::Element { component, .. } if *component == VIRTUAL_COLLECTION)
}

#[doc = include_str!("virtual_collection.doc/has_virtual_collection.md")]
#[must_use]
pub fn has_virtual_collection(nodes: &[ViewNode]) -> bool {
    nodes.iter().any(|node| match node {
        ViewNode::Element { children, .. } => {
            is_virtual_collection(node) || has_virtual_collection(children.nodes())
        }
        ViewNode::Text { .. } => false,
    })
}

#[doc = include_str!("virtual_collection.doc/realize_virtual_collections.md")]
#[must_use]
pub fn realize_virtual_collections<F>(view: View, window_len: &F) -> View
where
    F: Fn(f32) -> usize,
{
    if !has_virtual_collection(view.root().nodes()) {
        return view;
    }
    View::from_fragment(realize_fragment(view.into_fragment(), window_len))
}

#[doc = include_str!("virtual_collection.doc/realize_fragment.md")]
fn realize_fragment<F>(fragment: ViewFragment, window_len: &F) -> ViewFragment
where
    F: Fn(f32) -> usize,
{
    let mut realized: Vec<ViewNode> = Vec::with_capacity(fragment.len());
    for node in fragment.into_nodes() {
        if is_virtual_collection(&node) {
            let ViewNode::Element {
                props, children, ..
            } = node
            else {
                unreachable!("is_virtual_collection matched an Element");
            };
            let extent = virtual_row_extent(&props).unwrap_or(DEFAULT_VIRTUAL_ROW_EXTENT);
            // Realize the template once (so a nested marker is windowed too), then
            // clone it across the window. Each clone is one row of the window.
            let template = realize_fragment(children, window_len);
            let count = window_len(extent);
            for _ in 0..count {
                realized.extend(template.nodes().iter().cloned());
            }
        } else {
            realized.push(realize_node(node, window_len));
        }
    }
    ViewFragment::from_nodes(realized)
}

#[doc = include_str!("virtual_collection.doc/realize_node.md")]
fn realize_node<F>(node: ViewNode, window_len: &F) -> ViewNode
where
    F: Fn(f32) -> usize,
{
    match node {
        ViewNode::Element {
            component,
            key,
            props,
            events,
            children,
        } => ViewNode::Element {
            component,
            key,
            props,
            events,
            children: realize_fragment(children, window_len),
        },
        text @ ViewNode::Text { .. } => text,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_VIRTUAL_ROW_EXTENT, VIRTUAL_COLLECTION, VIRTUAL_ROW_EXTENT, has_virtual_collection,
        is_virtual_collection, realize_virtual_collections, virtual_row_extent,
    };
    use crate::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};
    use obscur_identity::ComponentId;

    const BOX: ComponentId = ComponentId::from_raw(0x00C0_0001);
    const ROW: ComponentId = ComponentId::from_raw(0x00C0_0009);

    /// A `<div>` wrapping one virtual collection of a single-row template.
    fn div_with_virtual(extent: f32) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(BOX).with_children(ViewFragment::from_nodes(vec![
                ViewNode::virtual_collection(
                    extent,
                    ViewFragment::from_nodes(vec![ViewNode::element(ROW)]),
                ),
            ])),
        ]))
    }

    /// The children of the first top-level node.
    fn kids(view: &View) -> Vec<ViewNode> {
        let ViewNode::Element { children, .. } = &view.root().nodes()[0] else {
            panic!("a div is an element");
        };
        children.nodes().to_vec()
    }

    #[test]
    fn the_marker_carries_its_extent_and_template() {
        let node = ViewNode::virtual_collection(
            18.0,
            ViewFragment::from_nodes(vec![ViewNode::element(ROW)]),
        );
        assert!(is_virtual_collection(&node));
        let ViewNode::Element {
            component,
            props,
            children,
            ..
        } = &node
        else {
            panic!("a marker is an element");
        };
        assert_eq!(*component, VIRTUAL_COLLECTION);
        assert_eq!(props[0].name(), VIRTUAL_ROW_EXTENT);
        assert_eq!(virtual_row_extent(props), Some(18.0));
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn realization_replaces_the_marker_with_a_window_of_that_length() {
        // The window resolver ignores the extent and always asks for 7 rows.
        let realized = realize_virtual_collections(div_with_virtual(24.0), &|_extent| 7);
        let rows = kids(&realized);
        assert_eq!(rows.len(), 7, "the marker became seven template clones");
        assert!(
            rows.iter()
                .all(|n| matches!(n, ViewNode::Element { component, .. } if *component == ROW)),
            "each realized row is a clone of the template"
        );
        assert!(
            !rows.iter().any(is_virtual_collection),
            "no marker survives realization"
        );
    }

    #[test]
    fn the_window_resolver_sees_the_authored_extent() {
        let seen = std::cell::Cell::new(None);
        let _ = realize_virtual_collections(div_with_virtual(31.0), &|extent| {
            seen.set(Some(extent));
            3
        });
        assert_eq!(seen.get(), Some(31.0));
    }

    #[test]
    fn a_zero_length_window_leaves_no_row_behind() {
        let realized = realize_virtual_collections(div_with_virtual(24.0), &|_| 0);
        assert!(
            kids(&realized).is_empty(),
            "the marker itself is gone, not emptied"
        );
    }

    #[test]
    fn a_view_with_no_marker_is_returned_unchanged() {
        let plain = View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::element(BOX)]));
        assert!(!has_virtual_collection(plain.root().nodes()));
        assert_eq!(realize_virtual_collections(plain.clone(), &|_| 5), plain);
    }

    #[test]
    fn the_default_extent_applies_when_the_marker_carries_none() {
        // A marker whose extent prop is absent falls back to the default.
        let node = ViewNode::element(VIRTUAL_COLLECTION)
            .with_children(ViewFragment::from_nodes(vec![ViewNode::element(ROW)]));
        let ViewNode::Element { props, .. } = &node else {
            panic!("element");
        };
        assert_eq!(virtual_row_extent(props), None);
        let view = View::from_fragment(ViewFragment::from_nodes(vec![node]));
        let seen = std::cell::Cell::new(None);
        let _ = realize_virtual_collections(view, &|extent| {
            seen.set(Some(extent));
            2
        });
        assert_eq!(seen.get(), Some(DEFAULT_VIRTUAL_ROW_EXTENT));
    }

    #[test]
    fn an_int_extent_prop_is_read_as_a_number() {
        let node = ViewNode::element(VIRTUAL_COLLECTION)
            .with_prop(ViewProperty::new(VIRTUAL_ROW_EXTENT, ViewValue::Int(20)));
        let ViewNode::Element { props, .. } = &node else {
            panic!("element");
        };
        assert_eq!(virtual_row_extent(props), Some(20.0));
    }
}

/// The registered builder name a virtual-list marker names, if it is one.
#[must_use]
pub fn virtual_list_id(props: &[ViewProperty]) -> Option<InternedSymbol> {
    props.iter().find_map(|p| {
        (p.name() == VIRTUAL_LIST_ID).then(|| match p.value() {
            ViewValue::Int(raw) => u32::try_from(*raw).ok().map(InternedSymbol::from_raw),
            _ => None,
        })?
    })
}

/// The scroll-reset token a marker carries, or [`None`] when it carries none.
///
/// A change in this value between frames is the app's signal to start the list at the
/// top; its absolute value is meaningless.
#[must_use]
pub fn virtual_scroll_reset(props: &[ViewProperty]) -> Option<u64> {
    props.iter().find_map(|p| {
        (p.name() == VIRTUAL_SCROLL_RESET).then(|| match p.value() {
            #[expect(
                clippy::cast_sign_loss,
                reason = "the token is an opaque generation counter; its bit pattern is all that matters"
            )]
            ViewValue::Int(token) => Some(*token as u64),
            _ => None,
        })?
    })
}

/// Every `(list id, scroll-reset token)` a document's virtual-list markers carry, so
/// the runtime can reset the scroll of a list whose token changed (spec §4.9).
///
/// Only markers that name a list *and* carry a token appear; a marker without a token
/// opts out of reset-on-change entirely and keeps its scroll across content edits.
#[must_use]
pub fn virtual_scroll_resets(view: &View) -> Vec<(InternedSymbol, u64)> {
    let mut out = Vec::new();
    collect_scroll_resets(view.root().nodes(), &mut out);
    out
}

/// Depth-first walk gathering the `(list id, token)` pairs [`virtual_scroll_resets`]
/// returns.
fn collect_scroll_resets(nodes: &[ViewNode], out: &mut Vec<(InternedSymbol, u64)>) {
    for node in nodes {
        let ViewNode::Element {
            component,
            props,
            children,
            ..
        } = node
        else {
            continue;
        };
        if *component == VIRTUAL_COLLECTION
            && let Some(id) = virtual_list_id(props)
            && let Some(token) = virtual_scroll_reset(props)
        {
            out.push((id, token));
        }
        collect_scroll_resets(children.nodes(), out);
    }
}

/// How many items the marker says exist.
#[must_use]
pub fn virtual_item_count(props: &[ViewProperty]) -> usize {
    props
        .iter()
        .find_map(|p| {
            (p.name() == VIRTUAL_ITEM_COUNT).then(|| match p.value() {
                ViewValue::Int(n) => usize::try_from(*n).ok(),
                _ => None,
            })?
        })
        .unwrap_or(0)
}

/// Replaces every virtual-list marker with its realized window (spec §4.9).
///
/// `window` is asked, per marker, for the half-open range of item indices the
/// viewport can show; `row` builds one item's node. The realized children are
/// **a leading spacer, the window's rows, then a trailing spacer**, each spacer
/// sized to the items it stands in for — so the container's content extent is the
/// true `count * row_extent` even though only the window exists as nodes. That is
/// what a scroll container needs in order to size a scrollbar honestly, and it is
/// exactly the half an app cannot supply for itself: to state the full height it
/// would have to build the full list, which is the cost virtualization exists to
/// avoid.
pub fn realize_virtual_lists<W, R>(view: View, window: &W, row: &mut R) -> View
where
    W: Fn(InternedSymbol, usize, f32) -> core::ops::Range<usize>,
    R: FnMut(InternedSymbol, usize) -> Option<ViewNode>,
{
    View::from_fragment(realize_list_fragment(view.into_fragment(), window, row))
}

/// A spacer standing in for `items` unbuilt rows of `extent` each.
fn spacer(list: InternedSymbol, items: usize, extent: f32) -> ViewNode {
    ViewNode::element(VIRTUAL_SPACER)
        .with_prop(ViewProperty::new(
            VIRTUAL_ROW_EXTENT,
            ViewValue::Float(f64::from(items as f32 * extent)),
        ))
        // Names the list this spacer belongs to, so the runtime can find the
        // container it was realized into and read that container's scroll offset.
        // The offset has to be the container's own — it is what the wheel, the
        // scrollbar and `scroll_by` all move — or the window and the scroll drift
        // apart and the rows slide out of view.
        .with_prop(ViewProperty::new(
            VIRTUAL_LIST_ID,
            ViewValue::Int(i64::from(list.raw())),
        ))
}

fn realize_list_fragment<W, R>(fragment: ViewFragment, window: &W, row: &mut R) -> ViewFragment
where
    W: Fn(InternedSymbol, usize, f32) -> core::ops::Range<usize>,
    R: FnMut(InternedSymbol, usize) -> Option<ViewNode>,
{
    let mut out = Vec::new();
    for node in fragment.into_nodes() {
        let ViewNode::Element {
            component, props, ..
        } = &node
        else {
            out.push(node);
            continue;
        };
        let Some(id) = (*component == VIRTUAL_COLLECTION)
            .then(|| virtual_list_id(props))
            .flatten()
        else {
            let ViewNode::Element {
                component,
                props,
                children,
                key,
                events,
            } = node
            else {
                unreachable!("matched an Element above");
            };
            out.push(ViewNode::Element {
                component,
                props,
                key,
                events,
                children: realize_list_fragment(children, window, row),
            });
            continue;
        };
        let count = virtual_item_count(props);
        let extent = virtual_row_extent(props).unwrap_or(DEFAULT_VIRTUAL_ROW_EXTENT);
        let range = window(id, count, extent);
        let first = range.start.min(count);
        let last = range.end.min(count);
        if first > 0 {
            out.push(spacer(id, first, extent));
        }
        for index in first..last {
            if let Some(built) = row(id, index) {
                out.push(built);
            }
        }
        if last < count {
            out.push(spacer(id, count - last, extent));
        }
    }
    ViewFragment::from_nodes(out)
}
