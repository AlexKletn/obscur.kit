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
    },
    #[doc = include_str!("props_error.doc/Type.md")]
    Type {
        #[doc = include_str!("props_error.doc/name_2.md")]
        name: InternedSymbol,
        #[doc = include_str!("props_error.doc/expected.md")]
        expected: &'static str,
    },
}

impl PropsError {
    #[doc = include_str!("props_error.doc/name_3.md")]
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
