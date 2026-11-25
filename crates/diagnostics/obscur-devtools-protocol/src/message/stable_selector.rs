/// Stable node selectors independent of styles and layout.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StableSelector {
    /// Exact test identifier.
    TestId(Box<str>),
    /// Exact automation identifier.
    AutomationId(Box<str>),
    /// Component ID. Multiple retained instances can match.
    Component(u64),
    /// Current runtime node ID.
    Node(u64),
}
