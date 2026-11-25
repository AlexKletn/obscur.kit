use crate::matching::StyleRule;
use crate::selector::SimpleSelector;

#[derive(Debug, Clone)]
pub(super) struct StylesheetData {
    pub(super) rules: Vec<StyleRule>,
    pub(super) sibling_targets: Vec<SimpleSelector>,
    pub(super) container_properties: Vec<crate::property::PropertyId>,
    pub(super) container_ranges: Vec<obscur_identity::InternedSymbol>,
    pub(super) container_references:
        Vec<(crate::property::PropertyId, obscur_identity::InternedSymbol)>,
    pub(super) container_ranges_use_ch: bool,
}
