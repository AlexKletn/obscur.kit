use obscur_style::cascade::CascadeOrigin;
use obscur_style::matching::StyleRule;

/// A rule that could produce one kind of generated decoration, carried with the
/// two things the cascade needs and the rule itself does not know: the origin of
/// the sheet it came from, and where its block sits in document order.
#[derive(Debug, Clone, Copy)]
pub(super) struct CandidateRule<'a> {
    /// The cascade origin of the sheet the rule was written in (§4.14).
    pub(super) origin: CascadeOrigin,
    /// The rule itself, borrowed from the sheet set this index was built over.
    pub(super) rule: &'a StyleRule,
    /// The document-order number of the rule's FIRST declaration, counted over every
    /// declaration of every rule in the sheet set — including the rules this index
    /// dropped. Declarations of a kept rule therefore land on exactly the numbers an
    /// unfiltered walk would have given them, so filtering cannot change a
    /// document-order tie-break.
    pub(super) order: u32,
}
