#[doc = include_str!("../../docs/api/matching/rule_index.doc/RuleCandidate.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleCandidate {
    sheet: u32,
    rule: u32,
    decl_start: u32,
    has_custom: bool,
}

impl RuleCandidate {
    pub(super) const fn new(sheet: u32, rule: u32, decl_start: u32, has_custom: bool) -> Self {
        Self {
            sheet,
            rule,
            decl_start,
            has_custom,
        }
    }

    /// Index of the sheet this rule lives in, within the set the index was built
    /// from.
    #[must_use]
    pub const fn sheet(self) -> usize {
        self.sheet as usize
    }

    /// Index of this rule within its sheet.
    #[must_use]
    pub const fn rule(self) -> usize {
        self.rule as usize
    }

    /// The `source_order` its first block declaration would have been given by a walk
    /// that visited every rule before it — that is, the sum of the block lengths of
    /// all preceding rules across all preceding sheets.
    #[must_use]
    pub const fn decl_start(self) -> u32 {
        self.decl_start
    }

    /// Whether the rule declares any custom property, so the custom-property pass can
    /// skip it without touching the rule at all.
    #[must_use]
    pub const fn has_custom(self) -> bool {
        self.has_custom
    }
}
