//! The cascade engine: [`Cascade`] resolves the winning declaration per
//! property (§4.14).
use obscur_identity::FastMap;

use crate::cascade::{CascadePriority, Declaration, ResolvedDeclarations};
use crate::property::PropertyId;
use std::collections::HashMap;

#[doc = include_str!("../../docs/api/cascade/cascade.doc/Cascade.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Cascade;

impl Cascade {
    /// Creates a cascade engine.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("../../docs/api/cascade/cascade.doc/resolve.md")]
    #[must_use]
    pub fn resolve(&self, declarations: &[Declaration]) -> ResolvedDeclarations {
        // For each property, remember the winning declaration's priority so we
        // can compare later candidates against it in a single pass.
        let mut winners: FastMap<PropertyId, (CascadePriority, &Declaration)> = HashMap::default();

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
        // The winning Inline declaration is deliberately FIRST: were the loop
        // merely keeping the last declaration it saw, the trailing Theme one
        // would take Color and this would pass for the wrong reason.
        let decls = [
            decl(
                PropertyId::Color,
                PropertyValue::Number(2.0),
                CascadeOrigin::Inline,
                ImportantFlag::Normal,
                spec(0, 0, 0),
                0,
            ),
            decl(
                PropertyId::Width,
                PropertyValue::Number(3.0),
                CascadeOrigin::GlobalApp,
                ImportantFlag::Normal,
                spec(0, 0, 0),
                1,
            ),
            decl(
                PropertyId::Color,
                PropertyValue::Number(1.0),
                CascadeOrigin::Theme,
                ImportantFlag::Normal,
                spec(9, 9, 9),
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
    fn later_lower_priority_declaration_loses_to_earlier_winner() {
        // The whole point of the priority comparison: source position alone must
        // never promote a declaration. Every one of these losers comes after its
        // winner, and each isolates a single axis of `CascadePriority`.
        let cases = [
            // Origin: Inline (winner, first) vs Theme (loser, later).
            (
                CascadeOrigin::Inline,
                ImportantFlag::Normal,
                spec(0, 0, 0),
                CascadeOrigin::Theme,
                ImportantFlag::Normal,
                spec(0, 0, 0),
            ),
            // Importance: authored `!important` beats a later normal of the
            // same origin and higher specificity.
            (
                CascadeOrigin::GlobalApp,
                ImportantFlag::Important,
                spec(0, 0, 1),
                CascadeOrigin::GlobalApp,
                ImportantFlag::Normal,
                spec(9, 9, 9),
            ),
            // Specificity: within one band, the more specific selector wins
            // even though the loser is later in source order.
            (
                CascadeOrigin::GlobalApp,
                ImportantFlag::Normal,
                spec(1, 0, 0),
                CascadeOrigin::GlobalApp,
                ImportantFlag::Normal,
                spec(0, 9, 9),
            ),
        ];
        for (win_origin, win_important, win_spec, lose_origin, lose_important, lose_spec) in cases {
            let decls = [
                decl(
                    PropertyId::Color,
                    PropertyValue::Number(1.0),
                    win_origin,
                    win_important,
                    win_spec,
                    0,
                ),
                decl(
                    PropertyId::Color,
                    PropertyValue::Number(2.0),
                    lose_origin,
                    lose_important,
                    lose_spec,
                    1,
                ),
            ];
            let resolved = Cascade::new().resolve(&decls);
            assert_eq!(
                resolved.get(PropertyId::Color),
                Some(&PropertyValue::Number(1.0)),
                "the later {lose_origin:?}/{lose_important:?}/{lose_spec:?} declaration must not \
                 displace the earlier {win_origin:?}/{win_important:?}/{win_spec:?} one"
            );
        }
    }

    #[test]
    fn runtime_override_beats_authored_important() {
        // Winner first, authored `!important` last, so passing cannot be an
        // artefact of position.
        let decls = [
            decl(
                PropertyId::Color,
                PropertyValue::Number(2.0),
                CascadeOrigin::RuntimeOverride,
                ImportantFlag::Normal,
                spec(0, 0, 0),
                0,
            ),
            decl(
                PropertyId::Color,
                PropertyValue::Number(1.0),
                CascadeOrigin::Inline,
                ImportantFlag::Important,
                spec(9, 9, 9),
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
        // order): the later of the two tied declarations must win
        // deterministically. A strictly weaker declaration trails both of them,
        // so "keeps the last seen" is pinned to the tie, not to the slice end.
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
            decl(
                PropertyId::Color,
                PropertyValue::Number(3.0),
                CascadeOrigin::SystemDefault,
                ImportantFlag::Normal,
                spec(9, 9, 9),
                9,
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
        // Order 6 beats order 5 at equal specificity, and the trailing order-7
        // declaration still loses because its specificity is lower: source order
        // is the tiebreaker, not the ranking.
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
            decl(
                PropertyId::Color,
                PropertyValue::Number(3.0),
                CascadeOrigin::GlobalApp,
                ImportantFlag::Normal,
                spec(1, 1, 0),
                7,
            ),
        ];
        let resolved = Cascade::new().resolve(&decls);
        assert_eq!(
            resolved.get(PropertyId::Color),
            Some(&PropertyValue::Number(2.0))
        );
    }
}
