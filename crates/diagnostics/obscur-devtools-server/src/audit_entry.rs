use obscur_devtools_protocol::message::{CommandId, CommandKind};

#[doc = include_str!("../docs/api/endpoint.doc/AuditEntry.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEntry {
    /// The id of the command that was audited.
    pub command: CommandId,
    /// The dangerous command body that was recorded.
    pub kind: CommandKind,
}

impl AuditEntry {
    /// Creates an audit entry.
    #[must_use]
    pub const fn new(command: CommandId, kind: CommandKind) -> Self {
        Self { command, kind }
    }
}
