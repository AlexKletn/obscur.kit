#![doc = include_str!("exported_style_part.doc/_module.md")]

use crate::scope::ScopeId;
use obscur_identity::InternedSymbol;

#[doc = include_str!("exported_style_part.doc/ExportedStylePart.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedStylePart {
    scope: ScopeId,
    name: InternedSymbol,
    states: Vec<InternedSymbol>,
}

impl ExportedStylePart {
    #[doc = include_str!("exported_style_part.doc/new.md")]
    #[must_use]
    pub fn new(scope: ScopeId, name: InternedSymbol, states: Vec<InternedSymbol>) -> Self {
        Self {
            scope,
            name,
            states,
        }
    }

    #[doc = include_str!("exported_style_part.doc/stateless.md")]
    #[must_use]
    pub fn stateless(scope: ScopeId, name: InternedSymbol) -> Self {
        Self::new(scope, name, Vec::new())
    }

    #[doc = include_str!("exported_style_part.doc/scope.md")]
    #[must_use]
    pub const fn scope(&self) -> ScopeId {
        self.scope
    }

    #[doc = include_str!("exported_style_part.doc/name.md")]
    #[must_use]
    pub const fn name(&self) -> InternedSymbol {
        self.name
    }

    #[doc = include_str!("exported_style_part.doc/states.md")]
    #[must_use]
    pub fn states(&self) -> &[InternedSymbol] {
        &self.states
    }

    #[doc = include_str!("exported_style_part.doc/exposes.md")]
    #[must_use]
    pub fn exposes(&self, state: InternedSymbol) -> bool {
        self.states.contains(&state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_round_trip() {
        let part = ExportedStylePart::new(
            ScopeId::from_raw(1),
            InternedSymbol::from_raw(2),
            vec![InternedSymbol::from_raw(3)],
        );
        assert_eq!(part.scope(), ScopeId::from_raw(1));
        assert_eq!(part.name(), InternedSymbol::from_raw(2));
        assert_eq!(part.states(), &[InternedSymbol::from_raw(3)]);
    }

    #[test]
    fn exposes_only_declared_states() {
        let part = ExportedStylePart::new(
            ScopeId::from_raw(1),
            InternedSymbol::from_raw(2),
            vec![InternedSymbol::from_raw(3)],
        );
        assert!(part.exposes(InternedSymbol::from_raw(3)));
        assert!(!part.exposes(InternedSymbol::from_raw(4)));
    }

    #[test]
    fn stateless_exposes_nothing() {
        let part = ExportedStylePart::stateless(ScopeId::from_raw(1), InternedSymbol::from_raw(2));
        assert!(part.states().is_empty());
        assert!(!part.exposes(InternedSymbol::from_raw(3)));
    }
}
