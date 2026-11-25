use obscur_style::matching::{RuleIndex, Stylesheet};

/// A [`RuleIndex`] this pass will build only if a node actually reads one.
///
/// Two node sets never do: a pass over a dirty set too small to repay the
/// build, and a pass whose every node sits inside a component boundary and so
/// takes the `::part()` path, which the index cannot answer for. Both were
/// paying for a structure nobody opened.
pub(super) struct LazyRuleIndex {
    built: Option<RuleIndex>,
    wanted: bool,
}

impl LazyRuleIndex {
    /// An index worth building, once something asks for it.
    pub(super) const fn wanted() -> Self {
        Self {
            built: None,
            wanted: true,
        }
    }

    /// No index: every node walks every rule, as it did before there was one.
    pub(super) const fn unwanted() -> Self {
        Self {
            built: None,
            wanted: false,
        }
    }

    /// The index, built on the first ask and kept for the rest of the pass.
    pub(super) fn get(&mut self, sheets: &[Stylesheet]) -> Option<&RuleIndex> {
        if !self.wanted {
            return None;
        }
        Some(self.built.get_or_insert_with(|| RuleIndex::build(sheets)))
    }
}
