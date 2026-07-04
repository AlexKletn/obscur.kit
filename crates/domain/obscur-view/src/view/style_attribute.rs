//! Well-known style attributes carried on element view nodes for CSS matching.
//!
//! CSS selectors match on an element's *type* (local name), its `id`, and its
//! `class` list (spec §4.13). Those live on a [`ViewNode`](super::ViewNode)
//! element as ordinary [`ViewProperty`] bindings under a small set of reserved
//! interned symbols, defined here so the whole toolkit agrees on them. The
//! reconciler reads them via [`StyleAttributes::extract`] and stores the result
//! on the retained node, where the selector matcher can see it.
//!
//! The reserved names occupy a low symbol namespace (`0x0000_00xx`), distinct
//! from the component/primitive prop namespaces (e.g. `0x00C1_xxxx`), so they
//! never collide with an ordinary prop.

use obscur_identity::InternedSymbol;

use super::{ViewProperty, ViewValue};

/// Reserved symbol for the element's `class` attribute — a whitespace-separated
/// list of class names (as [`ViewValue::Text`]).
pub const CLASS: InternedSymbol = InternedSymbol::from_raw(0x0000_0001);

/// Reserved symbol for the element's `id` attribute (as [`ViewValue::Text`]).
pub const ID: InternedSymbol = InternedSymbol::from_raw(0x0000_0002);

/// Reserved symbol for the element's *type* / local name — the tag a CSS type
/// selector matches (as [`ViewValue::Text`]).
pub const TYPE: InternedSymbol = InternedSymbol::from_raw(0x0000_0003);

/// The CSS-relevant identity of an element view node: its local name, `id`, and
/// class list, extracted from a node's [`ViewProperty`] bindings.
///
/// Names are kept as owned strings (not interned) because CSS selector names
/// are matched as strings — the authored stylesheet and the tree intern names
/// in separate tables, so string comparison is the only coherent match.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StyleAttributes {
    local_name: Option<Box<str>>,
    id: Option<Box<str>>,
    classes: Vec<Box<str>>,
}

impl StyleAttributes {
    /// Creates an empty attribute set (no local name, id, or classes).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Extracts the well-known style attributes from a node's properties.
    ///
    /// Reads the [`CLASS`], [`ID`], and [`TYPE`] reserved symbols; only
    /// [`ViewValue::Text`] values contribute (other value kinds are ignored).
    /// The `class` text is split on ASCII whitespace into individual class
    /// names, preserving order and dropping empties.
    #[must_use]
    pub fn extract(props: &[ViewProperty]) -> Self {
        let mut attributes = Self::new();
        for prop in props {
            let ViewValue::Text(text) = prop.value() else {
                continue;
            };
            match prop.name() {
                CLASS => {
                    attributes.classes =
                        text.split_ascii_whitespace().map(Into::into).collect();
                }
                ID => attributes.id = Some(text.clone()),
                TYPE => attributes.local_name = Some(text.clone()),
                _ => {}
            }
        }
        attributes
    }

    /// The element's local name (type), if set.
    #[must_use]
    pub fn local_name(&self) -> Option<&str> {
        self.local_name.as_deref()
    }

    /// The element's `id`, if set.
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// The element's class list (possibly empty).
    #[must_use]
    pub fn classes(&self) -> &[Box<str>] {
        &self.classes
    }

    /// Whether this element carries a given class.
    #[must_use]
    pub fn has_class(&self, class: &str) -> bool {
        self.classes.iter().any(|c| c.as_ref() == class)
    }

    /// Whether no style attribute is set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.local_name.is_none() && self.id.is_none() && self.classes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_prop(name: InternedSymbol, value: &str) -> ViewProperty {
        ViewProperty::new(name, ViewValue::Text(value.into()))
    }

    #[test]
    fn empty_when_no_attributes() {
        let attrs = StyleAttributes::extract(&[]);
        assert!(attrs.is_empty());
        assert_eq!(attrs.local_name(), None);
        assert_eq!(attrs.id(), None);
        assert!(attrs.classes().is_empty());
    }

    #[test]
    fn extracts_local_name_id_and_classes() {
        let props = [
            text_prop(TYPE, "container"),
            text_prop(ID, "main"),
            text_prop(CLASS, "counter  primary   large"),
        ];
        let attrs = StyleAttributes::extract(&props);
        assert_eq!(attrs.local_name(), Some("container"));
        assert_eq!(attrs.id(), Some("main"));
        let classes: Vec<&str> = attrs.classes().iter().map(AsRef::as_ref).collect();
        assert_eq!(classes, ["counter", "primary", "large"]);
        assert!(attrs.has_class("primary"));
        assert!(!attrs.has_class("missing"));
        assert!(!attrs.is_empty());
    }

    #[test]
    fn ignores_non_text_and_unknown_symbols() {
        let props = [
            ViewProperty::new(CLASS, ViewValue::Int(7)),
            ViewProperty::new(InternedSymbol::from_raw(0x00C1_0501), ViewValue::Text("w".into())),
        ];
        let attrs = StyleAttributes::extract(&props);
        assert!(attrs.is_empty());
    }

    #[test]
    fn blank_class_yields_no_classes() {
        let attrs = StyleAttributes::extract(&[text_prop(CLASS, "   ")]);
        assert!(attrs.classes().is_empty());
    }
}
