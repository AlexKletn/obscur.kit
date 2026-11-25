//! Contextual anchors for a diagnostic.

use obscur_identity::{NodeId, ResourceId, WindowId};

/// Optional identity context attached to a diagnostic so tools can point at the
/// exact node/resource/window involved (spec §1.6).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DiagnosticContext {
    /// Node the diagnostic concerns, if any.
    pub node: Option<NodeId>,
    /// Window the diagnostic concerns, if any.
    pub window: Option<WindowId>,
    /// Resource the diagnostic concerns, if any.
    pub resource: Option<ResourceId>,
}
