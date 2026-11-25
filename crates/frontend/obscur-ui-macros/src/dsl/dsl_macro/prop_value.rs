/// A resolved attribute value.
pub(super) enum PropValue {
    /// A bare attribute (`disabled`), meaning boolean-true.
    BareTrue,
    /// A literal string value.
    Text(String),
    /// A literal boolean value.
    Bool(bool),
    /// A literal integer value.
    Int(i64),
    /// A literal float value.
    Float(f64),
    /// A dynamic binding to a state/prop path (or stringified expression).
    Binding(String),
}
