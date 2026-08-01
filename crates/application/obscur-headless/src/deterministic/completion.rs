#![doc = include_str!("completion.doc/_module.md")]

use obscur_identity::OperationId;

#[doc = include_str!("completion.doc/Completion.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Completion<T> {
    #[doc = include_str!("completion.doc/operation.md")]
    operation: OperationId,
    #[doc = include_str!("completion.doc/value.md")]
    value: T,
}

impl<T> Completion<T> {
    #[doc = include_str!("completion.doc/new.md")]
    #[must_use]
    pub fn new(operation: OperationId, value: T) -> Self {
        Self { operation, value }
    }

    #[doc = include_str!("completion.doc/operation_2.md")]
    #[must_use]
    pub fn operation(&self) -> OperationId {
        self.operation
    }

    #[doc = include_str!("completion.doc/value_2.md")]
    #[must_use]
    pub fn value(&self) -> &T {
        &self.value
    }

    #[doc = include_str!("completion.doc/into_value.md")]
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
