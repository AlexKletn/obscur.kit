//! Virtual collections: the reserved marker a `<VirtualFor>` lowers to, and the
//! viewport-bounded realization that replaces it with a window of rows (spec
//! §2.11, §4.9).
//!
//! # What §4.9 asks for
//!
//! §2.11 names two collection constructs, `<For>` and `<VirtualFor>`, and they
//! are not two spellings of one thing. A `<VirtualFor>` is a *performance
//! contract*: §4.9 says it materializes a **window** of the collection rather
//! than the whole of it, widened by overscan, with the geometry of what is not
//! yet measured *estimated*. A `<For>` over a million rows builds a million
//! nodes; a `<VirtualFor>` over the same collection builds only the handful the
//! viewport can show.
//!
//! # Why a marker, and why realization runs in the runtime
//!
//! The window a `<VirtualFor>` realizes is a function of the **viewport** — how
//! tall it is, how far it has scrolled — and of an **estimated row extent**. A
//! *view builder* runs before layout and is handed neither, so the front end
//! cannot expand the window itself: it lowers `<VirtualFor>` to a single
//! [`VIRTUAL_COLLECTION`] marker node carrying the per-row template as its
//! children and the estimated row extent as a reserved property.
//!
//! The runtime, which owns the viewport, then substitutes each marker away —
//! exactly as [`project_slots`](super::project_slots) substitutes a slot outlet
//! — replacing it, *before the tree is reconciled*, with as many clones of its
//! template as the window holds. The window count comes from
//! [`obscur_layout::virtualization::VirtualLayout`], so it is bounded by the
//! viewport and independent of the collection's length. That count-independence
//! is the whole point of the construct, and is what a `<For>`, which splices its
//! template once and lets the runtime materialize one node per item, does not
//! have.
//!
//! # Why a reserved `ComponentId` and not a `ViewNode` variant
//!
//! Because a marker never becomes a retained node — realization substitutes it
//! away before reconcile, the same reason [`SLOT_OUTLET`](super::SLOT_OUTLET) is
//! a reserved id rather than a third enum arm. A `ViewNode` variant would add a
//! case to every exhaustive match over the enum to describe a node no consumer
//! of the retained tree can ever see.

use obscur_identity::{ComponentId, InternedSymbol, StableKey};

use super::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// Reserved [`ComponentId`] of a **virtual-collection marker** — the node a
/// `<VirtualFor>` lowers to, standing in for the window of rows that
/// realization materializes (spec §2.11, §4.9).
///
/// Nothing is ever registered under this id and no node ever retains it:
/// [`realize_virtual_collections`] replaces every marker with a viewport-bounded
/// window of its template before the output reaches the reconciler.
///
/// It sits alongside [`SLOT_OUTLET`](super::SLOT_OUTLET) in the reserved
/// `0x00B0_xxxx` range — below the primitive (`0x00C0_xxxx`) and widget
/// (`0x00D0_xxxx`) ranges and clear of the top-bit namespace a user component
/// hashes into — so it can alias neither a shipped component nor an author's.
pub const VIRTUAL_COLLECTION: ComponentId = ComponentId::from_raw(0x00B0_0002);

/// Reserved symbol for a virtual collection's **estimated row extent** — the
/// per-item size, in logical pixels along the scroll axis, the window is sized
/// against until real measurement refines it (spec §4.9 *estimated geometry*).
///
/// Like [`SLOT`](super::SLOT) it lives in the structural `0x0000_03xx`
/// namespace, distinct from the CSS-relevant reserved attributes
/// (`0x0000_000x`), the reserved events (`0x0000_010x`) and the stable selectors
/// (`0x0000_02xx`): it is a realization input, not a CSS selector input, so no
/// selector can match on it.
pub const VIRTUAL_ROW_EXTENT: InternedSymbol = InternedSymbol::from_raw(0x0000_0302);

/// Reserved symbol naming which registered row builder a virtual list realizes
/// through (spec §4.9).
///
/// The builder itself cannot live in the [`View`]: a `View` is plain data —
/// `Clone`, `Debug`, `PartialEq` — and a closure is none of those. So the view
/// carries only this name and the runtime holds the builder, which is what lets a
/// list of 10,000 entries put 10,000 *items* in the model while only ever building
/// the nodes the viewport can show.
/// A realized virtual list's spacer: an empty box standing in for the rows outside
/// the window, sized to exactly the space they would have taken.
///
/// It is what makes the container's content extent true without building the rows,
/// so the scrollbar is proportioned correctly and native scrolling covers the whole
/// list rather than only the part that happens to exist.
pub const VIRTUAL_SPACER: ComponentId = ComponentId::from_raw(0x00B0_0003);

pub const VIRTUAL_LIST_ID: InternedSymbol = InternedSymbol::from_raw(0x0000_0303);

/// Reserved symbol for a virtual list's **item count** — how many items exist,
/// not how many are built (spec §4.9).
///
/// This is the half an app-side virtualization cannot express. Because the runtime
/// knows it, it can give the scroll container the true content extent
/// (`count * row_extent`) while realizing only the window — so the scrollbar is
/// honest, the wheel is native, and nothing about paging is the app's arithmetic.
pub const VIRTUAL_ITEM_COUNT: InternedSymbol = InternedSymbol::from_raw(0x0000_0304);

/// The default estimated row extent, in logical pixels, a `<VirtualFor>` uses
/// until its rows are really measured (spec §4.9 *estimated geometry*).
///
/// A comfortable single-row height: enough that a normal viewport shows tens of
/// rows, which is the order [`realize_virtual_collections`] materializes.
pub const DEFAULT_VIRTUAL_ROW_EXTENT: f32 = 24.0;

/// The estimated row extent a marker's props carry, or [`None`] when it carries
/// none (or one that is not a number).
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

/// Whether this node is a virtual-collection marker (spec §2.11, §4.9).
///
/// The single predicate for "this node is a virtual collection".
/// [`realize_virtual_collections`] substitutes every node it answers `true` for;
/// like a slot outlet, such a node is never meant to reach the retained tree.
#[must_use]
pub fn is_virtual_collection(node: &ViewNode) -> bool {
    matches!(node, ViewNode::Element { component, .. } if *component == VIRTUAL_COLLECTION)
}

/// Whether `nodes`, or anything beneath them, contains a virtual-collection
/// marker (spec §2.11).
///
/// [`realize_virtual_collections`]'s early-out: a document that authors no
/// `<VirtualFor>` pays a single shallow walk and is returned untouched.
#[must_use]
pub fn has_virtual_collection(nodes: &[ViewNode]) -> bool {
    nodes.iter().any(|node| match node {
        ViewNode::Element { children, .. } => {
            is_virtual_collection(node) || has_virtual_collection(children.nodes())
        }
        ViewNode::Text { .. } => false,
    })
}

/// Realizes every virtual-collection marker in `view` into a viewport-bounded
/// window of rows (spec §2.11, §4.9).
///
/// Each marker is replaced, in place and in order, by `window_len(extent)`
/// clones of its template children, where `extent` is the marker's estimated row
/// extent (defaulting to [`DEFAULT_VIRTUAL_ROW_EXTENT`]). The caller supplies
/// `window_len` — the runtime computes it from the live viewport through
/// [`obscur_layout::virtualization::VirtualLayout`], which is what makes the
/// realized count bounded by the viewport and independent of the collection's
/// length.
///
/// `view` is returned untouched, without walking or rebuilding it, when it holds
/// no marker — which is every document that authors no `<VirtualFor>`.
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

/// [`realize_virtual_collections`] over one fragment.
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

/// [`realize_virtual_collections`] into one node's children. Text has none.
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
                // Key each row by its model index. The index IS the row's identity
                // here, and the runtime knows it, so an app never has to say so.
                // Without it the reconciler matches the window positionally, and a
                // scroll of one row reads as "every row's content changed" — every
                // row re-updates and its invalidation cascades to the siblings that
                // follow it, which is most of the list, on every frame of a scroll.
                // With it, a one-row scroll is one insert and one remove.
                out.push(built.with_key(StableKey::from(index as u64)));
            }
        }
        if last < count {
            out.push(spacer(id, count - last, extent));
        }
    }
    ViewFragment::from_nodes(out)
}
