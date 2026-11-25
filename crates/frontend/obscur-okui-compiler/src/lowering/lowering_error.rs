#[doc = include_str!("../../docs/api/lowering/ui_ir_lowering.doc/LoweringError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweringError {
    /// The document had no top-level elements to lower.
    EmptyDocument,
    /// An element carried no tag name; the byte offset locates it in the source.
    MissingTagName(u32),
    /// A `<For>`/`<VirtualFor>` loop lacked the mandatory stable `key={…}`
    /// (spec §2.11); the byte offset locates it in the source.
    MissingLoopKey(u32),
}

impl std::fmt::Display for LoweringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyDocument => write!(f, "cannot lower an empty document"),
            Self::MissingTagName(offset) => {
                write!(f, "element is missing a tag name at offset {offset}")
            }
            Self::MissingLoopKey(offset) => {
                write!(f, "loop is missing a mandatory `key` at offset {offset}")
            }
        }
    }
}

impl std::error::Error for LoweringError {}
