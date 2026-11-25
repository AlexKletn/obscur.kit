/// Which token terminated a collected declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DeclarationTerminator {
    Semicolon,
    Brace,
    Eof,
}
