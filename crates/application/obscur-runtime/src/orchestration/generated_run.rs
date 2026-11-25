use obscur_style::{cascade::ResolvedDeclarations, property::StyleColor};

#[doc = include_str!("../../docs/api/orchestration/generated_content.doc/GeneratedRun.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedRun {
    /// The literal text to paint.
    pub text: Box<str>,
    /// The resolved `color`, or [`None`] to inherit the node's ink.
    pub color: Option<StyleColor>,
    /// Cascaded declarations applied to a generated decoration box.
    pub declarations: ResolvedDeclarations,
}
