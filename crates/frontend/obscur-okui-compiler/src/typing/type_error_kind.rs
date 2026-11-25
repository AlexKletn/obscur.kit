/// The kind of failure a [`TypeError`](super::TypeError) reports (spec §2.9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeErrorKind {
    /// A value's type is not assignable to the expected type.
    Mismatch,
    /// Two operands cannot be compared with each other.
    NotComparable,
    /// A value used as a condition is not boolean-assignable.
    NotBoolean,
    /// A required type could not be determined.
    UnknownType,
}
