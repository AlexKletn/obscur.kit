#[doc = include_str!("../../docs/api/contract/component_surface.doc/PropTypeSurface.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PropTypeSurface {
    /// A string prop (`String`, `&str`, `Box<str>`, `Cow<str>`, …).
    String,
    /// An integer prop (`i8`…`i128`, `u8`…`u128`, `isize`, `usize`).
    Int,
    /// A floating-point prop (`f32`, `f64`).
    Float,
    /// A boolean prop (`bool`).
    Bool,
    /// A type the classifier does not recognise; never type-checked.
    #[default]
    Other,
}
