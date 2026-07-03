//! A style part a component exports across its scope boundary.
//!
//! Per §4.15, components may export style parts and states so that outside CSS
//! can target them (like `::part()`), and the host may override an exported
//! part. An [`ExportedStylePart`] is the style-side counterpart to a component's
//! exported-part contract: it names the exported part and the visual states it
//! exposes for the given scope. The actual selector matching against these parts
//! lives in the matcher / `obscur-css`.

use crate::scope::ScopeId;
use obscur_identity::InternedSymbol;

/// A style part exported across a scope boundary, with its exposed states
/// (§4.15).
///
/// The host may override an exported part, so this only *declares* the contract.
/// It owns a `Vec` of states, hence `Clone` rather than `Copy`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedStylePart {
    scope: ScopeId,
    name: InternedSymbol,
    states: Vec<InternedSymbol>,
}

impl ExportedStylePart {
    /// Creates an exported part named `name` in `scope`, exposing `states`.
    #[must_use]
    pub fn new(scope: ScopeId, name: InternedSymbol, states: Vec<InternedSymbol>) -> Self {
        Self {
            scope,
            name,
            states,
        }
    }

    /// Creates an exported part that exposes no states.
    #[must_use]
    pub fn stateless(scope: ScopeId, name: InternedSymbol) -> Self {
        Self::new(scope, name, Vec::new())
    }

    /// Returns the scope that exports this part.
    #[must_use]
    pub const fn scope(&self) -> ScopeId {
        self.scope
    }

    /// Returns the exported part's name.
    #[must_use]
    pub const fn name(&self) -> InternedSymbol {
        self.name
    }

    /// Returns the states this part exposes.
    #[must_use]
    pub fn states(&self) -> &[InternedSymbol] {
        &self.states
    }

    /// Returns `true` when this part exposes the given visual state.
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
