#![doc = include_str!("validation_result.doc/_module.md")]

use crate::validation::UiIrValidationError;

#[doc = include_str!("validation_result.doc/UiIrValidationResult.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UiIrValidationResult {
    errors: Vec<UiIrValidationError>,
    warnings: Vec<UiIrValidationError>,
}

impl UiIrValidationResult {
    #[doc = include_str!("validation_result.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    #[doc = include_str!("validation_result.doc/push_error.md")]
    pub fn push_error(&mut self, error: UiIrValidationError) {
        self.errors.push(error);
    }

    #[doc = include_str!("validation_result.doc/push_warning.md")]
    pub fn push_warning(&mut self, warning: UiIrValidationError) {
        self.warnings.push(warning);
    }

    #[doc = include_str!("validation_result.doc/is_valid.md")]
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    #[doc = include_str!("validation_result.doc/errors.md")]
    #[must_use]
    pub fn errors(&self) -> &[UiIrValidationError] {
        &self.errors
    }

    #[doc = include_str!("validation_result.doc/warnings.md")]
    #[must_use]
    pub fn warnings(&self) -> &[UiIrValidationError] {
        &self.warnings
    }

    #[doc = include_str!("validation_result.doc/error_count.md")]
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    #[doc = include_str!("validation_result.doc/merge.md")]
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
