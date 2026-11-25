use obscur_identity::NodeId;

#[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/Press.md")]
#[derive(Debug, Clone, Copy)]
pub(crate) struct Press {
    /// The interactive target the press landed on. `None` for a press on empty
    /// space, which is still an outstanding press — it simply anchors nothing, so
    /// it activates nothing on release and lights no `:active`.
    pub(crate) target: Option<NodeId>,
    /// The click count this press resolved to (`1`, `2`, or `3`), kept here so a
    /// release reports the count of **its own** press rather than whichever press
    /// happened to be the most recent.
    pub(crate) clicks: u32,
}
