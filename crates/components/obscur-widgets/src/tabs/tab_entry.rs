//! A tab label with optional persistent identity.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TabEntry {
    pub(super) id: Option<Box<str>>,
    pub(super) label: Box<str>,
}
