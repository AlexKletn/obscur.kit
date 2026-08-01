#![doc = include_str!("match_element.doc/_module.md")]

use super::pseudo_state::PseudoState;

#[doc = include_str!("match_element.doc/MatchElement.md")]
pub trait MatchElement {
    #[doc = include_str!("match_element.doc/local_name.md")]
    fn local_name(&self) -> Option<&str>;

    #[doc = include_str!("match_element.doc/id.md")]
    fn id(&self) -> Option<&str>;

    #[doc = include_str!("match_element.doc/has_class.md")]
    fn has_class(&self, class: &str) -> bool;

    #[doc = include_str!("match_element.doc/attribute.md")]
    fn attribute(&self, name: &str) -> Option<&str>;

    #[doc = include_str!("match_element.doc/pseudo_state.md")]
    fn pseudo_state(&self) -> PseudoState;

    #[doc = include_str!("match_element.doc/parent.md")]
    fn parent(&self) -> Option<Self>
    where
        Self: Sized;

    #[doc = include_str!("match_element.doc/previous_sibling_element.md")]
    fn previous_sibling_element(&self) -> Option<Self>
    where
        Self: Sized;

    #[doc = include_str!("match_element.doc/index_among_siblings.md")]
    fn index_among_siblings(&self) -> usize;

    #[doc = include_str!("match_element.doc/sibling_count.md")]
    fn sibling_count(&self) -> usize;

    #[doc = include_str!("match_element.doc/is_root.md")]
    fn is_root(&self) -> bool;

    #[doc = include_str!("match_element.doc/has_children.md")]
    fn has_children(&self) -> bool;

    #[doc = include_str!("match_element.doc/is_only_child.md")]
    fn is_only_child(&self) -> bool {
        self.sibling_count() == 1
    }

    #[doc = include_str!("match_element.doc/nth_of_type_index.md")]
    fn nth_of_type_index(&self) -> usize {
        self.index_among_siblings() + 1
    }

    #[doc = include_str!("match_element.doc/nth_last_of_type_index.md")]
    fn nth_last_of_type_index(&self) -> usize {
        self.sibling_count() - self.index_among_siblings()
    }
}
