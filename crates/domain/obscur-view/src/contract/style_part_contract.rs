//! `StylePartContract`: an exported style part and its states (spec §2.12).

use obscur_identity::InternedSymbol;

/// The exported contract for one style part of a component (spec §2.12).
///
/// A style part is a named, addressable region of a component that stylesheets
/// may target (for example a "track" or "thumb"). Alongside its name it
/// exports the set of visual states it can be in (for example "hover" or
/// "disabled"). Both the part name and every state are interned handles, so
/// styling references them by identity rather than by string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StylePartContract {
    /// Interned name of the exported style part.
    pub name: InternedSymbol,
    /// Interned names of the visual states this part can expose.
    pub states: Vec<InternedSymbol>,
}

impl StylePartContract {
    /// Creates a style part contract with `name` exposing `states`.
    #[must_use]
    pub fn new(name: InternedSymbol, states: Vec<InternedSymbol>) -> Self {
        Self { name, states }
    }

    /// Creates a style part contract that exposes no visual states.
    #[must_use]
    pub const fn stateless(name: InternedSymbol) -> Self {
        Self {
            name,
            states: Vec::new(),
        }
    }

    /// Returns whether the part exports the given visual `state`.
    #[must_use]
    pub fn exposes(&self, state: InternedSymbol) -> bool {
        self.states.contains(&state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stateless_part_exposes_nothing() {
        let part = StylePartContract::stateless(InternedSymbol::from_raw(9));
        assert!(part.states.is_empty());
        assert!(!part.exposes(InternedSymbol::from_raw(1)));
    }

    #[test]
    fn exposes_declared_state() {
        let hover = InternedSymbol::from_raw(1);
        let part = StylePartContract::new(InternedSymbol::from_raw(9), vec![hover]);
        assert!(part.exposes(hover));
        assert!(!part.exposes(InternedSymbol::from_raw(2)));
    }
}
