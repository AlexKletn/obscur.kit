//! Resolves a named placement edge after auto-repeat expansion.

use super::GridLine;

/// Matching boundaries of one named line on an axis containing an auto-repeat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepeatedGridLine {
    occurrence: std::num::NonZeroI32,
    before: Vec<usize>,
    pattern: Vec<usize>,
    after: Vec<usize>,
    count_implicit: bool,
    preferred: Option<Box<RepeatedGridLine>>,
}

impl RepeatedGridLine {
    /// Creates a line lookup from zero-based boundary indices.
    ///
    /// Before contains absolute indices, pattern is relative to one
    /// repetition, and after starts at the end of the repeat.
    #[must_use]
    pub fn new(
        occurrence: std::num::NonZeroI32,
        before: Vec<usize>,
        pattern: Vec<usize>,
        after: Vec<usize>,
        count_implicit: bool,
    ) -> Self {
        Self {
            occurrence,
            before,
            pattern,
            after,
            count_implicit,
            preferred: None,
        }
    }

    /// Prefers the area-edge spelling when it names a line in the expanded grid.
    #[must_use]
    pub fn with_preferred(mut self, preferred: Self) -> Self {
        self.preferred = Some(Box::new(preferred));
        self
    }

    pub(in crate::algorithm) fn resolve(
        &self,
        start: usize,
        pattern_len: usize,
        expanded_len: usize,
    ) -> Option<GridLine> {
        if let Some(line) = self
            .preferred
            .as_ref()
            .and_then(|lookup| lookup.resolve(start, pattern_len, expanded_len))
        {
            return Some(line);
        }
        let mut matches = self.before.clone();
        if !self.pattern.is_empty() {
            let copies = expanded_len.checked_div(pattern_len).unwrap_or(0);
            for copy in 0..copies {
                let offset = start.checked_add(copy * pattern_len)?;
                for &line in &self.pattern {
                    matches.push(offset.checked_add(line)?);
                }
            }
        }
        if !self.after.is_empty() {
            let repeat_end = start.checked_add(expanded_len)?;
            for &line in &self.after {
                matches.push(repeat_end.checked_add(line)?);
            }
        }
        matches.sort_unstable();
        matches.dedup();
        let occurrence = self.occurrence.get();
        let offset = usize::try_from(occurrence.unsigned_abs())
            .ok()?
            .checked_sub(1)?;
        if offset >= matches.len() && self.count_implicit {
            return Some(GridLine::outside_explicit(
                u32::try_from(offset - matches.len() + 1).ok()?,
                occurrence < 0,
            ));
        }
        let index = if occurrence < 0 {
            matches.len().checked_sub(offset + 1)?
        } else {
            offset
        };
        let line = i32::try_from(*matches.get(index)?).ok()?.checked_add(1)?;
        Some(GridLine::Line(line))
    }
}

#[cfg(test)]
mod tests {
    use super::RepeatedGridLine;
    use std::num::NonZeroI32;

    #[test]
    fn overflowing_auto_repeat_boundaries_do_not_wrap_to_a_grid_line() {
        let pattern =
            RepeatedGridLine::new(NonZeroI32::new(1).unwrap(), vec![], vec![1], vec![], false);
        assert_eq!(pattern.resolve(usize::MAX, 1, 1), None);

        let suffix =
            RepeatedGridLine::new(NonZeroI32::new(1).unwrap(), vec![], vec![], vec![1], false);
        assert_eq!(suffix.resolve(usize::MAX, 1, 1), None);
    }
}
