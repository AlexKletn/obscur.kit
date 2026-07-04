//! The abstract element the matcher walks: [`MatchElement`].

use super::pseudo_state::PseudoState;

/// An element the selector matcher can navigate and inspect.
///
/// This is the seam that keeps `obscur-style` a pure domain crate: the matcher
/// operates only through this trait, so a retained-tree node (or any other
/// element graph) can implement it in a later sub-step without this crate
/// depending on the tree. Names and attributes are exchanged as `&str` — not
/// interned symbols — because CSS and the tree intern into different tables;
/// see the crate's interner-coherence note.
///
/// Navigation methods ([`parent`](Self::parent),
/// [`previous_sibling_element`](Self::previous_sibling_element)) return owned
/// `Self` values: implementors are expected to be cheap `Copy`/handle/cursor
/// types over a shared arena rather than owning the node data.
///
/// Only element siblings are considered "siblings" here; text and other
/// non-element children are excluded from indices and counts.
pub trait MatchElement {
    /// Returns the element's local/type name (e.g. `"div"`), or [`None`] if it
    /// has none. Type-selector comparison is ASCII case-insensitive.
    fn local_name(&self) -> Option<&str>;

    /// Returns the element's id, or [`None`] if unset.
    fn id(&self) -> Option<&str>;

    /// Returns `true` if the element carries the given class.
    fn has_class(&self, class: &str) -> bool;

    /// Returns the value of the named attribute, or [`None`] if absent.
    fn attribute(&self, name: &str) -> Option<&str>;

    /// Returns the element's active interaction state bitset.
    fn pseudo_state(&self) -> PseudoState;

    /// Returns the element's parent, or [`None`] if it is the root or detached.
    fn parent(&self) -> Option<Self>
    where
        Self: Sized;

    /// Returns the immediately preceding element sibling, or [`None`].
    fn previous_sibling_element(&self) -> Option<Self>
    where
        Self: Sized;

    /// Returns the element's 0-based index among its element siblings.
    fn index_among_siblings(&self) -> usize;

    /// Returns the total number of element siblings including this element
    /// (always at least `1`).
    fn sibling_count(&self) -> usize;

    /// Returns `true` if the element is the root of its tree (`:root`).
    fn is_root(&self) -> bool;

    /// Returns `true` if the element has any element children; `false` means the
    /// element is `:empty`.
    fn has_children(&self) -> bool;

    /// Returns `true` if this element is the only element among its siblings
    /// (`:only-child`).
    ///
    /// Defaults to `sibling_count() == 1`.
    fn is_only_child(&self) -> bool {
        self.sibling_count() == 1
    }

    /// Returns the element's 1-based index among same-type siblings, counting
    /// from the front (`:nth-of-type`).
    ///
    /// The default does not distinguish types and returns the overall 1-based
    /// index; implementors that track element types should override it.
    fn nth_of_type_index(&self) -> usize {
        self.index_among_siblings() + 1
    }

    /// Returns the element's 1-based index among same-type siblings, counting
    /// from the back (`:nth-last-of-type`).
    ///
    /// The default does not distinguish types and returns the overall 1-based
    /// index from the back; implementors that track element types should
    /// override it.
    fn nth_last_of_type_index(&self) -> usize {
        self.sibling_count() - self.index_among_siblings()
    }
}
