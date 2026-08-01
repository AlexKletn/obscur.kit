#![doc = include_str!("cascade.doc/_module.md")]

use crate::cascade::{CascadePriority, Declaration, ResolvedDeclarations};
use crate::property::PropertyId;
use std::collections::HashMap;

#[doc = include_str!("cascade.doc/Cascade.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Cascade;

impl Cascade {
    #[doc = include_str!("cascade.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("cascade.doc/resolve.md")]
    #[must_use]
    pub fn resolve(&self, declarations: &[Declaration]) -> ResolvedDeclarations {
        // For each property, remember the winning declaration's priority so we
        // can compare later candidates against it in a single pass.
        let mut winners: HashMap<PropertyId, (CascadePriority, &Declaration)> = HashMap::new();

        for declaration in declarations {
            let priority = declaration.priority();
            let property = declaration.property();
            match winners.get(&property) {
                // Replace on greater-or-equal so that, on an exact priority tie,
                // the later declaration in source wins (deterministic).
                Some((best, _)) if priority < *best => {}
                _ => {
                    winners.insert(property, (priority, declaration));
                }
            }
        }

        let mut resolved = ResolvedDeclarations::new();
        for (property, (priority, declaration)) in winners {
            // The priority travels with the value: a logical property and its
            // physical counterpart cascade under different keys but compete, and
            // only the computed-value step knows the `direction` that pairs them.
            resolved.insert_with_priority(property, declaration.value().clone(), priority);
        }
        resolved
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cascade::{CascadeOrigin, ImportantFlag};
    use crate::property::PropertyValue;
    use crate::selector::SelectorSpecificity;

    fn decl(
        property: PropertyId,
        value: PropertyValue,
        origin: CascadeOrigin,
        important: ImportantFlag,
        specificity: SelectorSpecificity,
        source_order: u32,
    ) -> Declaration {
        Declaration::new(
            property,
            value,
            important,
            origin,
            specificity,
            source_order,
            None,
        )
    }

    fn spec(a: u16, b: u16, c: u16) -> SelectorSpecificity {
        SelectorSpecificity::new(a, b, c)
    }

    #[test]
    fn empty_input_resolves_empty() {
        let resolved = Cascade::new().resolve(&[]);
        assert!(resolved.is_empty());
    }

    #[test]
    fn single_declaration_wins() {
        let decls = [decl(
            PropertyId::Color,
            PropertyValue::Number(1.0),
            CascadeOrigin::Theme,
            ImportantFlag::Normal,
            spec(0, 0, 0),
            0,
        )];
        let resolved = Cascade::new().resolve(&decls);
        assert_eq!(
            resolved.get(PropertyId::Color),
            Some(&PropertyValue::Number(1.0))
        );
    }

    #[test]
    fn higher_priority_declaration_wins_per_property() {
        // Normal Theme (loses) vs normal Inline (wins) for Color; a separate
        // Width declaration is unaffected.
        let decls = [
            decl(
                PropertyId::Color,
                PropertyValue::Number(1.0),
                CascadeOrigin::Theme,
                ImportantFlag::Normal,
                spec(9, 9, 9),
                0,
            ),
            decl(
                PropertyId::Color,
                PropertyValue::Number(2.0),
                CascadeOrigin::Inline,
                ImportantFlag::Normal,
                spec(0, 0, 0),
                1,
            ),
            decl(
                PropertyId::Width,
                PropertyValue::Number(3.0),
                CascadeOrigin::GlobalApp,
                ImportantFlag::Normal,
                spec(0, 0, 0),
                2,
            ),
        ];
        let resolved = Cascade::new().resolve(&decls);
        assert_eq!(
            resolved.get(PropertyId::Color),
            Some(&PropertyValue::Number(2.0))
        );
        assert_eq!(
            resolved.get(PropertyId::Width),
            Some(&PropertyValue::Number(3.0))
        );
        assert_eq!(resolved.len(), 2);
    }

    #[test]
    fn runtime_override_beats_authored_important() {
        let decls = [
            decl(
                PropertyId::Color,
                PropertyValue::Number(1.0),
                CascadeOrigin::Inline,
                ImportantFlag::Important,
                spec(9, 9, 9),
                0,
            ),
            decl(
                PropertyId::Color,
                PropertyValue::Number(2.0),
                CascadeOrigin::RuntimeOverride,
                ImportantFlag::Normal,
                spec(0, 0, 0),
                1,
            ),
        ];
        let resolved = Cascade::new().resolve(&decls);
        assert_eq!(
            resolved.get(PropertyId::Color),
            Some(&PropertyValue::Number(2.0))
        );
    }

    #[test]
    fn exact_tie_keeps_last_seen() {
        // Identical priority (same origin, importance, specificity, source
        // order): the later declaration in the slice must win deterministically.
        let decls = [
            decl(
                PropertyId::Color,
                PropertyValue::Number(1.0),
                CascadeOrigin::Theme,
                ImportantFlag::Normal,
                spec(0, 0, 0),
                0,
            ),
            decl(
                PropertyId::Color,
                PropertyValue::Number(2.0),
                CascadeOrigin::Theme,
                ImportantFlag::Normal,
                spec(0, 0, 0),
                0,
            ),
        ];
        let resolved = Cascade::new().resolve(&decls);
        assert_eq!(
            resolved.get(PropertyId::Color),
            Some(&PropertyValue::Number(2.0))
        );
    }

    #[test]
    fn later_source_order_wins_within_band() {
        let decls = [
            decl(
                PropertyId::Color,
                PropertyValue::Number(1.0),
                CascadeOrigin::GlobalApp,
                ImportantFlag::Normal,
                spec(1, 1, 1),
                5,
            ),
            decl(
                PropertyId::Color,
                PropertyValue::Number(2.0),
                CascadeOrigin::GlobalApp,
                ImportantFlag::Normal,
                spec(1, 1, 1),
                6,
            ),
        ];
        let resolved = Cascade::new().resolve(&decls);
        assert_eq!(
            resolved.get(PropertyId::Color),
            Some(&PropertyValue::Number(2.0))
        );
    }
}
