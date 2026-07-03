//! Whether a declaration carries `!important`: [`ImportantFlag`] (§4.14).

/// Whether a declaration is `!important` (§4.14).
///
/// Within the same origin band, an `!important` declaration outranks a normal
/// one, so the type derives [`Ord`] with [`ImportantFlag::Normal`] <
/// [`ImportantFlag::Important`] to match. The full interleaving of importance,
/// origin and the runtime-override layer is computed by
/// [`crate::cascade::CascadePriority`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ImportantFlag {
    /// A normal (non-`!important`) declaration.
    Normal,
    /// An `!important` declaration.
    Important,
}

impl ImportantFlag {
    /// Returns `true` when this declaration is `!important`.
    #[must_use]
    pub const fn is_important(self) -> bool {
        matches!(self, Self::Important)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn important_outranks_normal() {
        assert!(ImportantFlag::Important > ImportantFlag::Normal);
    }

    #[test]
    fn is_important_predicate() {
        assert!(ImportantFlag::Important.is_important());
        assert!(!ImportantFlag::Normal.is_important());
    }
}
