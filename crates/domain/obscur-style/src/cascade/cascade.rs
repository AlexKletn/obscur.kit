//! The cascade engine: [`Cascade`] resolves the winning declaration per
//! property (§4.14).

use crate::cascade::{CascadePriority, Declaration, ResolvedDeclarations};
use crate::property::PropertyId;
use std::collections::HashMap;

/// The cascade engine (§4.14).
///
/// Given the declarations that matched a node, [`Cascade::resolve`] picks, for
/// each [`PropertyId`], the declaration with the greatest
/// [`CascadePriority`] and returns its value.
///
/// The engine is stateless; it is a zero-sized handle whose only job is to host
/// the resolution algorithm.
///
/// # Cascaded, not computed
///
/// [`Cascade::resolve`] produces the **cascaded value** per property: the raw
/// winning value straight from the source declaration. Turning a cascaded value
/// into a **computed value** — resolving the CSS-wide keywords
/// (`inherit` / `initial` / `unset`), inheriting from the parent element, and
/// substituting `var()` custom-property references — is the job of the next
/// sub-step (computed style) and is deliberately **not** done here.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Cascade;

impl Cascade {
    /// Creates a cascade engine.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Resolves the winning **cascaded value** for each property.
    ///
    /// For every [`PropertyId`] present among `declarations`, the returned
    /// [`ResolvedDeclarations`] holds the value of the declaration with the
    /// greatest [`CascadePriority`] (see the type-level docs for the exact
    /// §4.14 ordering).
    ///
    /// Resolution is deterministic. When two declarations for the same property
    /// compare *exactly* equal in priority — which cannot happen when
    /// `source_order` is unique within a band — the later one in `declarations`
    /// is kept.
    ///
    /// Only known [`PropertyId`] declarations participate; custom-property
    /// (`--*`) resolution is out of scope for this sub-step. No inheritance and
    /// no CSS-wide-keyword resolution is applied here — the returned values are
    /// cascaded values, not computed values.
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
        for (property, (_, declaration)) in winners {
            resolved.insert(property, declaration.value().clone());
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
