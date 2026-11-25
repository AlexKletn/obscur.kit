/// What an authored `<slot …/>` names its outlet — the value of its `name`
/// attribute, classified (spec §2.12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutletName<'a> {
    /// `name="header"` — a literal, and the only accepted form.
    Literal(&'a str),
    /// `name={expr}` — a name that does not exist until runtime.
    Dynamic,
    /// No `name` attribute at all.
    Missing,
}
