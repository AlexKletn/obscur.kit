//! A single controlled async completion (spec §7.4, §3.3).

use obscur_identity::OperationId;

#[doc = include_str!("../../docs/api/deterministic/completion.doc/Completion.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Completion<T> {
    /// The operation this completion resolves.
    operation: OperationId,
    /// The delivered result value.
    value: T,
}

impl<T> Completion<T> {
    /// Creates a completion pairing `operation` with its result `value`.
    #[must_use]
    pub fn new(operation: OperationId, value: T) -> Self {
        Self { operation, value }
    }

    /// Returns the operation this completion resolves.
    #[must_use]
    pub fn operation(&self) -> OperationId {
        self.operation
    }

    /// Returns a reference to the delivered result value.
    #[must_use]
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Consumes the completion, returning its result value.
    #[must_use]
    pub fn into_value(self) -> T {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_expose_operation_and_value() {
        let operation = OperationId::from_raw(7);
        let completion = Completion::new(operation, 42u32);

        assert_eq!(completion.operation(), operation);
        assert_eq!(*completion.value(), 42);
        assert_eq!(completion.into_value(), 42);
    }
}
