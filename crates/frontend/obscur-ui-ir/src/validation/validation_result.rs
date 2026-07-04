//! `UiIrValidationResult`: the accumulated outcome of validating a UI IR
//! document (spec §2.5 validation).

use crate::validation::UiIrValidationError;

/// The accumulated outcome of a validation pass over a UI IR document
/// (spec §2.5).
///
/// A result collects two buckets of [`UiIrValidationError`]: fatal `errors` and
/// non-fatal `warnings`. A freshly [`new`](UiIrValidationResult::new) (or
/// [`default`](Default::default)) result is empty, and an empty error bucket
/// means [`is_valid`](UiIrValidationResult::is_valid) — warnings never affect
/// validity. Results compose via [`merge`](UiIrValidationResult::merge), letting
/// a walk fold child outcomes into a parent's.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UiIrValidationResult {
    errors: Vec<UiIrValidationError>,
    warnings: Vec<UiIrValidationError>,
}

impl UiIrValidationResult {
    /// Creates an empty result — no errors and no warnings, hence valid.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Records a fatal error, making the result invalid.
    pub fn push_error(&mut self, error: UiIrValidationError) {
        self.errors.push(error);
    }

    /// Records a non-fatal warning; validity is unaffected.
    pub fn push_warning(&mut self, warning: UiIrValidationError) {
        self.warnings.push(warning);
    }

    /// Returns whether the document is valid — i.e. no errors were recorded.
    /// Warnings are permitted.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns the recorded errors.
    #[must_use]
    pub fn errors(&self) -> &[UiIrValidationError] {
        &self.errors
    }

    /// Returns the recorded warnings.
    #[must_use]
    pub fn warnings(&self) -> &[UiIrValidationError] {
        &self.warnings
    }

    /// Returns the number of recorded errors.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Absorbs another result's errors and warnings into this one.
    pub fn merge(&mut self, other: UiIrValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_and_new_are_empty_and_valid() {
        let result = UiIrValidationResult::new();
        assert!(result.is_valid());
        assert_eq!(result.error_count(), 0);
        assert!(result.errors().is_empty());
        assert!(result.warnings().is_empty());
        assert_eq!(result, UiIrValidationResult::default());
    }

    #[test]
    fn push_error_makes_invalid() {
        let mut result = UiIrValidationResult::new();
        result.push_error(UiIrValidationError::DuplicateNodeId(1));
        assert!(!result.is_valid());
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.errors(), &[UiIrValidationError::DuplicateNodeId(1)]);
    }

    #[test]
    fn push_warning_keeps_valid() {
        let mut result = UiIrValidationResult::new();
        result.push_warning(UiIrValidationError::EmptyElementTag(2));
        assert!(result.is_valid());
        assert_eq!(result.warnings().len(), 1);
        assert_eq!(result.error_count(), 0);
    }

    #[test]
    fn merge_absorbs_errors_and_warnings() {
        let mut a = UiIrValidationResult::new();
        a.push_error(UiIrValidationError::DuplicateNodeId(1));
        a.push_warning(UiIrValidationError::EmptyElementTag(2));

        let mut b = UiIrValidationResult::new();
        b.push_error(UiIrValidationError::TextNodeHasChildren(3));
        b.push_warning(UiIrValidationError::EmptyElementTag(4));

        a.merge(b);
        assert_eq!(a.error_count(), 2);
        assert_eq!(a.warnings().len(), 2);
        assert!(!a.is_valid());
    }
}
