use obscur_identity::InternedSymbol;

/// A literal or custom reference used in a container style query.
#[derive(Debug, Clone, PartialEq)]
pub enum StyleQueryValue<T> {
    /// An operand in the grammar of the query.
    Literal(T),
    /// A custom property read on the selected container.
    Variable {
        /// The custom property symbol.
        name: InternedSymbol,
        /// The operand to try if the reference is missing or unresolved.
        fallback: Option<Box<Self>>,
    },
}

impl<T> StyleQueryValue<T> {
    pub(super) fn variables(&self) -> impl Iterator<Item = InternedSymbol> + '_ {
        self.chain().filter_map(|part| match part {
            Self::Variable { name, .. } => Some(*name),
            Self::Literal(_) => None,
        })
    }

    pub(super) fn terminal_literal(&self) -> Option<&T> {
        match self.chain().last()? {
            Self::Literal(value) => Some(value),
            Self::Variable { .. } => None,
        }
    }

    pub(super) fn resolve_with<'a>(
        &'a self,
        lookup: impl Fn(InternedSymbol) -> Option<Option<&'a T>>,
    ) -> Option<&'a T> {
        for part in self.chain() {
            match part {
                Self::Literal(value) => return Some(value),
                Self::Variable { name, .. } => {
                    // A missing value advances the fallback. An unusable
                    // resolved value terminates the comparison.
                    if let Some(value) = lookup(*name)? {
                        return Some(value);
                    }
                }
            }
        }
        None
    }

    fn chain(&self) -> impl Iterator<Item = &Self> {
        std::iter::successors(Some(self), |part| match part {
            Self::Variable { fallback, .. } => fallback.as_deref(),
            Self::Literal(_) => None,
        })
    }
}
