use obscur_identity::InternedSymbol;

#[doc = include_str!("../../docs/api/runtime/canvas_painters.doc/PainterKey.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PainterKey {
    /// A painter a node named with its own symbol constant.
    Symbol(InternedSymbol),
    /// A painter named in CSS by `-obscur-painter`.
    Name(Box<str>),
}
