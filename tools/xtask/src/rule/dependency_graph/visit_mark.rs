/// DFS visitation state.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum VisitMark {
    /// Unvisited.
    White,
    /// On the current DFS stack.
    Gray,
    /// Fully explored.
    Black,
}
