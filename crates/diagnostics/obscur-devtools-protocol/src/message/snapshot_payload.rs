#[doc = include_str!("../../docs/api/message/snapshot_message.doc/SnapshotPayload.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SnapshotPayload {
    /// A short, human-readable description of the snapshot.
    pub description: Box<str>,
    /// The number of entities summarized (e.g. nodes, styles, resources).
    pub entity_count: u64,
    /// Stable ids of the summarized entities, when the server enumerates them
    /// (may be empty).
    pub entity_ids: Vec<u64>,
    /// An optional detailed, structured rendering of the target (e.g. the whole
    /// tree with per-node geometry and styles) — empty by default. Lets a rich
    /// host carry a full inspection an external agent can read, while the counts
    /// above stay a cheap summary (spec §7.6/§7.8).
    pub detail: Box<str>,
}

impl SnapshotPayload {
    /// Creates a snapshot payload with no detail.
    #[must_use]
    pub fn new(description: impl Into<Box<str>>, entity_count: u64, entity_ids: Vec<u64>) -> Self {
        Self {
            description: description.into(),
            entity_count,
            entity_ids,
            detail: Box::default(),
        }
    }

    /// Creates a payload carrying only a description.
    #[must_use]
    pub fn summary(description: impl Into<Box<str>>) -> Self {
        Self {
            description: description.into(),
            entity_count: 0,
            entity_ids: Vec::new(),
            detail: Box::default(),
        }
    }

    /// Attaches a detailed structured rendering, returning the payload (fluent).
    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<Box<str>>) -> Self {
        self.detail = detail.into();
        self
    }
}
