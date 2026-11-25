use obscur_style::matching::Stylesheet;

#[doc = include_str!("../../docs/api/runtime/cascade_sheets.doc/AuthorSheets.md")]
pub(crate) struct AuthorSheets(pub(super) Vec<Stylesheet>);
