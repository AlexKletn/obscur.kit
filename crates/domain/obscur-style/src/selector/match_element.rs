//! The abstract element the matcher walks: [`MatchElement`].

use super::pseudo_state::PseudoState;

#[doc = include_str!("../../docs/api/selector/match_element.doc/MatchElement.md")]
pub trait MatchElement {
    /// Returns the element's local/type name (e.g. `"div"`), or [`None`] if it
    /// has none. Type-selector comparison is ASCII case-insensitive.
    fn local_name(&self) -> Option<&str>;

    /// Returns the element's id, or [`None`] if unset.
    fn id(&self) -> Option<&str>;

    /// Returns `true` if the element carries the given class.
    fn has_class(&self, class: &str) -> bool;

    #[doc = include_str!("../../docs/api/selector/match_element.doc/enumerate_classes.md")]
    fn enumerate_classes(&self, visit: &mut dyn FnMut(&str)) -> bool {
        let _ = visit;
        false
    }

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

    #[doc = include_str!("../../docs/api/selector/match_element.doc/is_only_child.md")]
    fn is_only_child(&self) -> bool {
        self.sibling_count() == 1
    }

    #[doc = include_str!("../../docs/api/selector/match_element.doc/nth_of_type_index.md")]
    fn nth_of_type_index(&self) -> usize {
        self.index_among_siblings() + 1
    }

    #[doc = include_str!("../../docs/api/selector/match_element.doc/nth_last_of_type_index.md")]
    fn nth_last_of_type_index(&self) -> usize {
        self.sibling_count() - self.index_among_siblings()
    }
}
