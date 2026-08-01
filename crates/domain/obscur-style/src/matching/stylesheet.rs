#![doc = include_str!("stylesheet.doc/_module.md")]

use crate::cascade::CascadeOrigin;
use crate::matching::StyleRule;

#[doc = include_str!("stylesheet.doc/Stylesheet.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Stylesheet {
    origin: CascadeOrigin,
    rules: Vec<StyleRule>,
}

impl Stylesheet {
    #[doc = include_str!("stylesheet.doc/new.md")]
    #[must_use]
    pub const fn new(origin: CascadeOrigin, rules: Vec<StyleRule>) -> Self {
        Self { origin, rules }
    }

    #[doc = include_str!("stylesheet.doc/origin.md")]
    #[must_use]
    pub const fn origin(&self) -> CascadeOrigin {
        self.origin
    }

    #[doc = include_str!("stylesheet.doc/rules.md")]
    #[must_use]
    pub fn rules(&self) -> &[StyleRule] {
        &self.rules
    }

    #[doc = include_str!("stylesheet.doc/push.md")]
    pub fn push(&mut self, rule: StyleRule) {
        self.rules.push(rule);
    }
}

impl Default for Stylesheet {
    #[doc = include_str!("stylesheet.doc/default.md")]
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
