//! The per-node style dependency record: [`StyleDependency`] (§4.3, §4.14).
//!
//! When a node's computed style is resolved it consumes a few typed inputs. This
//! record captures *which* of those inputs the node actually read, so a later
//! change to one of them can decide precisely whether the node is affected —
//! invalidating only affected consumers (§4.14) rather than everything. It is the
//! dependency edge that replaces scattered ad-hoc dirty flags (§4.3).

use crate::computed::StyleDiff;
use crate::invalidation::StyleInvalidationReason;
use crate::property::InheritedProperty;
use crate::selector::PseudoState;
use obscur_identity::InternedSymbol;
use std::collections::HashSet;

/// Records what a single node's computed style depended on (§4.3).
///
/// A node's style depends on: whether it inherited any value from its parent
/// ([`inherits_from_parent`](Self::inherits_from_parent)), which custom
/// properties it read ([`custom_properties`](Self::custom_properties)), and which
/// interaction states its matching depends on
/// ([`pseudo_states`](Self::pseudo_states)). A later change consults this record
/// via the `affected_by_*` methods to decide whether the node must be restyled;
/// those methods are the gate that keeps invalidation confined to affected
/// consumers (§4.14).
///
/// This is the per-node dependency record that replaces ad-hoc dirty flags: the
/// runtime keeps one alongside each node's cached style, not a bag of booleans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleDependency {
    inherits_from_parent: bool,
    custom_properties: HashSet<InternedSymbol>,
    pseudo_states: PseudoState,
}

impl StyleDependency {
    /// Creates an empty dependency record: no inheritance, no custom properties,
    /// no pseudo-state dependence.
    ///
    /// Build it up with the `with_*`/`add_*` methods as resolution discovers what
    /// the node actually consumed.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inherits_from_parent: false,
            custom_properties: HashSet::new(),
            pseudo_states: PseudoState::empty(),
        }
    }

    /// Sets whether the node computed at least one inherited/default-inherited
    /// value from its parent, returning `self` for builder-style chaining.
    #[must_use]
    pub fn with_inherits_from_parent(mut self, inherits: bool) -> Self {
        self.inherits_from_parent = inherits;
        self
    }

    /// Sets the interaction states the node's matching depends on, returning
    /// `self` for builder-style chaining.
    #[must_use]
    pub fn with_pseudo_states(mut self, states: PseudoState) -> Self {
        self.pseudo_states = states;
        self
    }

    /// Records that the node read the custom property `name` (e.g. via `var()`),
    /// returning `self` for builder-style chaining.
    #[must_use]
    pub fn with_custom_property(mut self, name: InternedSymbol) -> Self {
        self.custom_properties.insert(name);
        self
    }

    /// Records that the node read the custom property `name` (e.g. via `var()`).
    pub fn add_custom_property(&mut self, name: InternedSymbol) {
        self.custom_properties.insert(name);
    }

    /// Records that the node computed at least one value inherited from its
    /// parent.
    pub fn set_inherits_from_parent(&mut self, inherits: bool) {
        self.inherits_from_parent = inherits;
    }

    /// Records the interaction states the node's matching depends on.
    pub fn set_pseudo_states(&mut self, states: PseudoState) {
        self.pseudo_states = states;
    }

    /// Returns whether the node inherited any value from its parent.
    #[must_use]
    pub fn inherits_from_parent(&self) -> bool {
        self.inherits_from_parent
    }

    /// Returns the set of custom properties the node read.
    #[must_use]
    pub fn custom_properties(&self) -> &HashSet<InternedSymbol> {
        &self.custom_properties
    }

    /// Returns whether the node read any custom property.
    #[must_use]
    pub fn reads_custom_properties(&self) -> bool {
        !self.custom_properties.is_empty()
    }

    /// Returns the interaction states the node's matching depends on.
    #[must_use]
    pub fn pseudo_states(&self) -> PseudoState {
        self.pseudo_states
    }

    /// Decides whether a change to the *parent's* computed style affects this
    /// node, returning the typed reason if so (§4.14).
    ///
    /// The node is affected when either:
    /// - it inherited from its parent and `parent_diff` changed at least one
    ///   property that inherits by default (checked with
    ///   [`InheritedProperty::of`] over
    ///   [`StyleDiff::changed_properties`]) — yields
    ///   [`StyleInvalidationReason::ParentStyleChanged`]; or
    /// - it reads custom properties and `parent_diff` reports a custom-property
    ///   change ([`StyleDiff::custom_properties_changed`]) — yields
    ///   [`StyleInvalidationReason::CustomPropertyChanged`].
    ///
    /// The inherited-property check is preferred when both apply. Returns `None`
    /// when the node consumed nothing the parent changed, so an unaffected node is
    /// never restyled.
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

    /// Returns whether a change to the node's active pseudo-states affects it.
    ///
    /// True when the flags that `changed` toggled intersect the states this
    /// node's matching depends on; a state the node never matched on cannot alter
    /// its style.
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
        new.set(PropertyId::Color, ComputedValue::color(StyleColor::TRANSPARENT));
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
        assert_eq!(dep.affected_by_parent_change(&diff_changing_inherited()), None);
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
        new.set(PropertyId::Color, ComputedValue::color(StyleColor::TRANSPARENT));
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
