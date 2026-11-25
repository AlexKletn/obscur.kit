/// Which `ChildContract` variant a `#[contract(children = ...)]` selects.
#[derive(Debug, Clone, Copy)]
pub(super) enum ChildrenKind {
    /// Any child component is accepted.
    Any,
    /// No children are accepted.
    None,
}
