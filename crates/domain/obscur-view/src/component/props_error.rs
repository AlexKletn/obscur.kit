#![doc = include_str!("props_error.doc/_module.md")]

use core::fmt;

use obscur_identity::InternedSymbol;

#[doc = include_str!("props_error.doc/PropsError.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropsError {
    #[doc = include_str!("props_error.doc/Missing.md")]
    Missing {
        #[doc = include_str!("props_error.doc/name.md")]
        name: InternedSymbol,
        #[doc = include_str!("props_error.doc/label.md")]
        label: &'static str,
    },
    #[doc = include_str!("props_error.doc/Type.md")]
    Type {
        #[doc = include_str!("props_error.doc/name_2.md")]
        name: InternedSymbol,
        #[doc = include_str!("props_error.doc/label_2.md")]
        label: &'static str,
        #[doc = include_str!("props_error.doc/expected.md")]
        expected: &'static str,
    },
}

impl PropsError {
    #[doc = include_str!("props_error.doc/missing_2.md")]
    #[must_use]
    pub const fn missing(name: InternedSymbol, label: &'static str) -> Self {
        Self::Missing { name, label }
    }

    #[doc = include_str!("props_error.doc/wrong_type.md")]
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

    #[doc = include_str!("props_error.doc/name_3.md")]
    #[must_use]
    pub const fn name(self) -> InternedSymbol {
        match self {
            Self::Missing { name, .. } | Self::Type { name, .. } => name,
        }
    }

    #[doc = include_str!("props_error.doc/label_3.md")]
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
                write!(f, "required property {} is missing", Named(*name, label))
            }
            Self::Type {
                name,
                label,
                expected,
            } => {
                write!(
                    f,
                    "property {} could not be read as {expected}",
                    Named(*name, label)
                )
            }
        }
    }
}

#[doc = include_str!("props_error.doc/Named.md")]
struct Named<'a>(InternedSymbol, &'a str);

impl fmt::Display for Named<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The number is kept even when the name is known: it is what every other
        // report about this property prints, and a reader correlating two of
        // them needs the two to agree on something.
        if self.1.is_empty() {
            write!(f, "#{}", self.0.raw())
        } else {
            write!(f, "`{}` (#{})", self.1, self.0.raw())
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
