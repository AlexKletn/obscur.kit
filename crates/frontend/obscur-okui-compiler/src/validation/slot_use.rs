/// What a member's `slot="…"` attribute assigns it to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotUse<'a> {
    /// A literal slot name, which is the only kind a contract can be checked
    /// against.
    Named(&'a str),
    /// `slot={expr}`. A slot is matched against a declaration at *compile* time,
    /// so a name that only exists at runtime cannot be checked against one — and
    /// nothing downstream would route it either.
    Dynamic,
}
