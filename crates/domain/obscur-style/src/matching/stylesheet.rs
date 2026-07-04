//! One cascade origin's worth of authored style rules: [`Stylesheet`] (§4.14).

use crate::cascade::CascadeOrigin;
use crate::matching::StyleRule;

/// A collection of [`StyleRule`]s that all share a single [`CascadeOrigin`]
/// (§4.14).
///
/// A stylesheet represents exactly one origin band: the system-default sheet,
/// the theme sheet, the global-app sheet, a component/module sheet, or the
/// inline sheet. A complete application therefore has several stylesheets, one
/// per origin; the [`StyleMatcher`](crate::matching::StyleMatcher) accepts an
/// ordered slice of them (low origin → high origin) and stamps each rule's
/// declarations with this origin during matching.
#[derive(Debug, Clone, PartialEq)]
pub struct Stylesheet {
    origin: CascadeOrigin,
    rules: Vec<StyleRule>,
}

impl Stylesheet {
    /// Creates a stylesheet for `origin` from its rules in document order.
    #[must_use]
    pub const fn new(origin: CascadeOrigin, rules: Vec<StyleRule>) -> Self {
        Self { origin, rules }
    }

    /// Returns the cascade origin every rule in this sheet belongs to.
    #[must_use]
    pub const fn origin(&self) -> CascadeOrigin {
        self.origin
    }

    /// Returns the sheet's rules in document order.
    #[must_use]
    pub fn rules(&self) -> &[StyleRule] {
        &self.rules
    }

    /// Appends a rule to the end of the sheet (later in document order).
    pub fn push(&mut self, rule: StyleRule) {
        self.rules.push(rule);
    }
}

impl Default for Stylesheet {
    /// An empty stylesheet in the [`CascadeOrigin::GlobalApp`] origin.
    fn default() -> Self {
        Self::new(CascadeOrigin::GlobalApp, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matching::BlockDeclaration;
    use crate::property::{PropertyId, PropertyValue};
    use crate::selector::{ComplexSelector, CompoundSelector, SelectorList, SimpleSelector};

    fn rule() -> StyleRule {
        StyleRule::new(
            SelectorList::new(vec![ComplexSelector::new(
                CompoundSelector::new(vec![SimpleSelector::Type("div".into())]),
                Vec::new(),
            )]),
            vec![BlockDeclaration::normal(
                PropertyId::Opacity,
                PropertyValue::Number(1.0),
            )],
        )
    }

    #[test]
    fn default_is_empty_global_app() {
        let sheet = Stylesheet::default();
        assert_eq!(sheet.origin(), CascadeOrigin::GlobalApp);
        assert!(sheet.rules().is_empty());
    }

    #[test]
    fn new_and_accessors() {
        let sheet = Stylesheet::new(CascadeOrigin::Theme, vec![rule()]);
        assert_eq!(sheet.origin(), CascadeOrigin::Theme);
        assert_eq!(sheet.rules().len(), 1);
    }

    #[test]
    fn push_appends_rules() {
        let mut sheet = Stylesheet::new(CascadeOrigin::Inline, Vec::new());
        sheet.push(rule());
        sheet.push(rule());
        assert_eq!(sheet.rules().len(), 2);
    }
}
