//! `PropsError`: why a node's properties could not become a component's typed props.

mod property_name;

use core::fmt;

use property_name::PropertyName;

use obscur_identity::InternedSymbol;

#[doc = include_str!("../../docs/api/component/props_error.doc/PropsError.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropsError {
    /// A required property was absent (or explicitly
    /// [`Null`](crate::view::ViewValue::Null), which is *absence of a value*).
    Missing {
        /// The property that was required but not supplied.
        name: InternedSymbol,
        #[doc = include_str!("../../docs/api/component/props_error.doc/label.md")]
        label: &'static str,
    },
    /// A property was present but its value could not become the field's type —
    /// e.g. a `Text` where a `bool` was wanted, or an integer outside the
    /// field's range.
    Type {
        /// The property whose value did not fit.
        name: InternedSymbol,
        #[doc = include_str!("../../docs/api/component/props_error.doc/label_2.md")]
        label: &'static str,
        /// The type that was wanted, as
        /// [`FromViewValue::TYPE_NAME`](crate::component::FromViewValue::TYPE_NAME).
        expected: &'static str,
    },
}

impl PropsError {
    /// A required property was not supplied, named both ways.
    #[must_use]
    pub const fn missing(name: InternedSymbol, label: &'static str) -> Self {
        Self::Missing { name, label }
    }

    /// A property was supplied but could not be read as the type the field wants.
    #[must_use]
    pub const fn wrong_type(
        name: InternedSymbol,
        label: &'static str,
        expected: &'static str,
    ) -> Self {
        Self::Type {
            name,
            label,
            expected,
        }
    }

    /// The property this error is about.
    #[must_use]
    pub const fn name(self) -> InternedSymbol {
        match self {
            Self::Missing { name, .. } | Self::Type { name, .. } => name,
        }
    }

    /// The property's name in words, or `""` when it was decoded by bare symbol.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Missing { label, .. } | Self::Type { label, .. } => label,
        }
    }
}

impl fmt::Display for PropsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { name, label } => {
                write!(
                    f,
                    "required property {} is missing",
                    PropertyName(*name, label)
                )
            }
            Self::Type {
                name,
                label,
                expected,
            } => {
                write!(
                    f,
                    "property {} could not be read as {expected}",
                    PropertyName(*name, label)
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
        assert_eq!(PropsError::missing(symbol(), "step").name(), symbol());
        assert_eq!(
            PropsError::wrong_type(symbol(), "step", "bool").name(),
            symbol()
        );
    }

    #[test]
    fn display_names_the_property_and_the_wanted_type() {
        let missing = PropsError::missing(symbol(), "").to_string();
        assert!(missing.contains("#7"), "{missing}");
        let mismatch = PropsError::wrong_type(symbol(), "", "bool").to_string();
        assert!(
            mismatch.contains("#7") && mismatch.contains("bool"),
            "{mismatch}"
        );
    }

    /// The whole point of the label: a reader can search for the name.
    ///
    /// An `InternedSymbol` is an FNV hash with no reverse table, so a report
    /// carrying only the number told an application author nothing about which
    /// property their component refused (audit 1-B).
    #[test]
    fn a_labelled_property_is_named_in_words() {
        let missing = PropsError::missing(symbol(), "key_id").to_string();
        assert!(missing.contains("`key_id`"), "{missing}");
        // And still carries the number, so it correlates with every other
        // report about the same property.
        assert!(missing.contains("#7"), "{missing}");

        let mismatch = PropsError::wrong_type(symbol(), "key_id", "bool").to_string();
        assert!(
            mismatch.contains("`key_id`") && mismatch.contains("bool"),
            "{mismatch}"
        );
    }

    #[test]
    fn an_unlabelled_property_falls_back_to_the_number_alone() {
        // Decoding by bare symbol is still legal — there is simply no literal
        // to print, and a stray pair of empty backticks would read as a name.
        assert_eq!(
            PropsError::missing(symbol(), "").to_string(),
            "required property #7 is missing"
        );
    }
}
