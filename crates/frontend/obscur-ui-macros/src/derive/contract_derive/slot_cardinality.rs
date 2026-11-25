use proc_macro2::TokenStream;
use quote::quote;

/// Which `SlotCardinality` variant a `#[contract(slot(cardinality = ...))]`
/// selects.
#[derive(Debug, Clone, Copy)]
pub(super) enum SlotCardinality {
    /// Exactly one child.
    One,
    /// Zero or one child.
    Optional,
    /// Any number of children.
    Many,
}

impl SlotCardinality {
    /// The `obscur_view::contract::SlotCardinality` variant this spells.
    pub(super) fn variant(self) -> TokenStream {
        match self {
            Self::One => quote!(ExactlyOne),
            Self::Optional => quote!(Optional),
            Self::Many => quote!(Many),
        }
    }
}
