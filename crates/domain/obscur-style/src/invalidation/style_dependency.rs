#![doc = include_str!("style_dependency.doc/_module.md")]

use crate::computed::StyleDiff;
use crate::invalidation::StyleInvalidationReason;
use crate::property::InheritedProperty;
use crate::selector::PseudoState;
use obscur_identity::{FastSet, InternedSymbol};
use std::collections::HashSet;

#[doc = include_str!("style_dependency.doc/StyleDependency.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleDependency {
    inherits_from_parent: bool,
    custom_properties: FastSet<InternedSymbol>,
    pseudo_states: PseudoState,
}

impl StyleDependency {
    #[doc = include_str!("style_dependency.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            inherits_from_parent: false,
            custom_properties: HashSet::default(),
            pseudo_states: PseudoState::empty(),
        }
    }

    #[doc = include_str!("style_dependency.doc/with_inherits_from_parent.md")]
    #[must_use]
    pub fn with_inherits_from_parent(mut self, inherits: bool) -> Self {
        self.inherits_from_parent = inherits;
        self
    }

    #[doc = include_str!("style_dependency.doc/with_pseudo_states.md")]
    #[must_use]
    pub fn with_pseudo_states(mut self, states: PseudoState) -> Self {
        self.pseudo_states = states;
        self
    }

    #[doc = include_str!("style_dependency.doc/with_custom_property.md")]
    #[must_use]
    pub fn with_custom_property(mut self, name: InternedSymbol) -> Self {
        self.custom_properties.insert(name);
        self
    }

    #[doc = include_str!("style_dependency.doc/add_custom_property.md")]
    pub fn add_custom_property(&mut self, name: InternedSymbol) {
        self.custom_properties.insert(name);
    }

    #[doc = include_str!("style_dependency.doc/set_inherits_from_parent.md")]
    pub fn set_inherits_from_parent(&mut self, inherits: bool) {
        self.inherits_from_parent = inherits;
    }

    #[doc = include_str!("style_dependency.doc/set_pseudo_states.md")]
    pub fn set_pseudo_states(&mut self, states: PseudoState) {
        self.pseudo_states = states;
    }

    #[doc = include_str!("style_dependency.doc/inherits_from_parent.md")]
    #[must_use]
    pub fn inherits_from_parent(&self) -> bool {
        self.inherits_from_parent
    }

    #[doc = include_str!("style_dependency.doc/custom_properties.md")]
    #[must_use]
    pub fn custom_properties(&self) -> &FastSet<InternedSymbol> {
        &self.custom_properties
    }

    #[doc = include_str!("style_dependency.doc/reads_custom_properties.md")]
    #[must_use]
    pub fn reads_custom_properties(&self) -> bool {
        !self.custom_properties.is_empty()
    }

    #[doc = include_str!("style_dependency.doc/pseudo_states.md")]
    #[must_use]
    pub fn pseudo_states(&self) -> PseudoState {
        self.pseudo_states
    }

    #[doc = include_str!("style_dependency.doc/affected_by_parent_change.md")]
    #[must_use]
    pub fn affected_by_parent_change(
        &self,
        parent_diff: &StyleDiff,
    ) -> Option<StyleInvalidationReason> {
        if self.inherits_from_parent
            && parent_diff
                .changed_properties()
                .iter()
                .any(|&property| InheritedProperty::of(property).is_inherited())
        {
            return Some(StyleInvalidationReason::ParentStyleChanged);
        }
        if self.reads_custom_properties() && parent_diff.custom_properties_changed() {
            return Some(StyleInvalidationReason::CustomPropertyChanged);
        }
        None
    }

    #[doc = include_str!("style_dependency.doc/affected_by_pseudo_change.md")]
    #[must_use]
    pub fn affected_by_pseudo_change(&self, changed: PseudoState) -> bool {
        (self.pseudo_states.bits() & changed.bits()) != 0
    }
}

impl Default for StyleDependency {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::computed::{ComputedStyle, ComputedValue};
    use crate::property::{PropertyId, PropertyValue, StyleColor};

    fn diff_changing_inherited() -> StyleDiff {
        // `color` inherits by default.
        let mut old = ComputedStyle::new();
        old.set(PropertyId::Color, ComputedValue::color(StyleColor::BLACK));
        let mut new = ComputedStyle::new();
        new.set(
            PropertyId::Color,
            ComputedValue::color(StyleColor::TRANSPARENT),
        );
        StyleDiff::between(&old, &new)
    }

    fn diff_changing_non_inherited() -> StyleDiff {
        // `opacity` does not inherit.
        let mut old = ComputedStyle::new();
        old.set(PropertyId::Opacity, ComputedValue::number(1.0));
        let mut new = ComputedStyle::new();
        new.set(PropertyId::Opacity, ComputedValue::number(0.5));
        StyleDiff::between(&old, &new)
    }

    fn diff_changing_custom() -> StyleDiff {
        let name = InternedSymbol::from_raw(7);
        let mut old = ComputedStyle::new();
        old.set_custom_property(name, PropertyValue::Number(1.0));
        let mut new = ComputedStyle::new();
        new.set_custom_property(name, PropertyValue::Number(2.0));
        StyleDiff::between(&old, &new)
    }

    #[test]
    fn inheriting_node_affected_by_inherited_change() {
        let dep = StyleDependency::new().with_inherits_from_parent(true);
        assert_eq!(
            dep.affected_by_parent_change(&diff_changing_inherited()),
            Some(StyleInvalidationReason::ParentStyleChanged)
        );
    }

    #[test]
    fn inheriting_node_not_affected_by_non_inherited_change() {
        let dep = StyleDependency::new().with_inherits_from_parent(true);
        assert_eq!(
            dep.affected_by_parent_change(&diff_changing_non_inherited()),
            None
        );
    }

    #[test]
    fn non_inheriting_node_not_affected_by_inherited_change() {
        let dep = StyleDependency::new().with_inherits_from_parent(false);
        assert_eq!(
            dep.affected_by_parent_change(&diff_changing_inherited()),
            None
        );
    }

    #[test]
    fn custom_property_reader_affected_by_custom_change() {
        let dep = StyleDependency::new().with_custom_property(InternedSymbol::from_raw(7));
        assert_eq!(
            dep.affected_by_parent_change(&diff_changing_custom()),
            Some(StyleInvalidationReason::CustomPropertyChanged)
        );
    }

    #[test]
    fn non_reader_not_affected_by_custom_change() {
        let dep = StyleDependency::new();
        assert_eq!(dep.affected_by_parent_change(&diff_changing_custom()), None);
    }

    #[test]
    fn inherited_change_takes_precedence_over_custom() {
        // Both an inherited property and a custom property changed; the inherited
        // reason wins.
        let name = InternedSymbol::from_raw(7);
        let mut old = ComputedStyle::new();
        old.set(PropertyId::Color, ComputedValue::color(StyleColor::BLACK));
        old.set_custom_property(name, PropertyValue::Number(1.0));
        let mut new = ComputedStyle::new();
        new.set(
            PropertyId::Color,
            ComputedValue::color(StyleColor::TRANSPARENT),
        );
        new.set_custom_property(name, PropertyValue::Number(2.0));
        let diff = StyleDiff::between(&old, &new);

        let dep = StyleDependency::new()
            .with_inherits_from_parent(true)
            .with_custom_property(name);
        assert_eq!(
            dep.affected_by_parent_change(&diff),
            Some(StyleInvalidationReason::ParentStyleChanged)
        );
    }

    #[test]
    fn affected_by_pseudo_change_intersects() {
        let dep = StyleDependency::new()
            .with_pseudo_states(PseudoState::HOVER.insert(PseudoState::FOCUS));
        // Overlaps on HOVER.
        assert!(dep.affected_by_pseudo_change(PseudoState::HOVER));
        // Overlaps on FOCUS.
        assert!(dep.affected_by_pseudo_change(PseudoState::FOCUS.insert(PseudoState::ACTIVE)));
        // No overlap.
        assert!(!dep.affected_by_pseudo_change(PseudoState::ACTIVE));
        // Empty change never affects.
        assert!(!dep.affected_by_pseudo_change(PseudoState::empty()));
    }

    #[test]
    fn no_pseudo_dependence_never_affected() {
        let dep = StyleDependency::new();
        assert!(!dep.affected_by_pseudo_change(PseudoState::HOVER));
    }
}
