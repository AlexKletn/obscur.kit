//! `PropsError`: why a node's properties could not become a component's typed props.

use core::fmt;

use obscur_identity::InternedSymbol;

/// Why decoding a node's [`ViewProperty`](crate::view::ViewProperty) slice into a
/// component's typed [`Props`](crate::component::Component::Props) failed
/// (spec §3.1).
///
/// Decoding is the boundary where untyped, authored data becomes typed inputs, so
/// its failures are *data* errors, not bugs: a `.okui` author can write a missing
/// or wrongly-typed attribute, and the runtime must report that rather than panic.
/// Both variants name the property by its [`InternedSymbol`] — the same symbol the
/// view tree carries — so a caller can map it back to a name for a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropsError {
    /// A required property was absent (or explicitly
    /// [`Null`](crate::view::ViewValue::Null), which is *absence of a value*).
    Missing {
        /// The property that was required but not supplied.
        name: InternedSymbol,
    },
    /// A property was present but its value could not become the field's type —
    /// e.g. a `Text` where a `bool` was wanted, or an integer outside the
    /// field's range.
    Type {
        /// The property whose value did not fit.
        name: InternedSymbol,
        /// The type that was wanted, as
        /// [`FromViewValue::TYPE_NAME`](crate::component::FromViewValue::TYPE_NAME).
        expected: &'static str,
    },
}

impl PropsError {
    /// The property this error is about.
    #[must_use]
    pub const fn name(self) -> InternedSymbol {
        match self {
            Self::Missing { name } | Self::Type { name, .. } => name,
        }
    }
}

impl fmt::Display for PropsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { name } => {
                write!(f, "required property #{} is missing", name.raw())
            }
            Self::Type { name, expected } => {
                write!(
                    f,
                    "property #{} could not be read as {expected}",
                    name.raw()
                )
            }
        }
    }
}

impl core::error::Error for PropsError {}

#[cfg(test)]
mod tests {
    use super::PropsError;
    use obscur_identity::InternedSymbol;

    fn symbol() -> InternedSymbol {
        InternedSymbol::from_raw(7)
    }

    #[test]
    fn both_variants_report_their_property() {
        assert_eq!(PropsError::Missing { name: symbol() }.name(), symbol());
        assert_eq!(
            PropsError::Type {
                name: symbol(),
                expected: "bool"
            }
            .name(),
            symbol()
        );
    }

    #[test]
    fn display_names_the_property_and_the_wanted_type() {
        let missing = PropsError::Missing { name: symbol() }.to_string();
        assert!(missing.contains("#7"), "{missing}");
        let mismatch = PropsError::Type {
            name: symbol(),
            expected: "bool",
        }
        .to_string();
        assert!(
            mismatch.contains("#7") && mismatch.contains("bool"),
            "{mismatch}"
        );
    }
}
