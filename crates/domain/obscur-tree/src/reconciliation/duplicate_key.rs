use obscur_identity::{NodeId, StableKey};

/// One sibling group that gave the same key to more than one desired child.
///
/// Keyed identity assumes a key names **one** child (spec §2.4). When several share
/// it, the matcher's key index keeps the first and the rest find no partner: they are
/// inserted as new while the previous ones are removed, every frame, which makes the
/// whole group churn — a full restyle and a full layout each time. It is silent
/// otherwise, so the runtime turns this into a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateKey {
    /// The parent whose children collided.
    pub parent: NodeId,
    /// The key more than one child claimed.
    pub key: StableKey,
    /// How many children claimed it.
    pub count: usize,
}
