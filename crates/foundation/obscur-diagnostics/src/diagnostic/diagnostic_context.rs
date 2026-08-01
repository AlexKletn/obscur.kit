#![doc = include_str!("diagnostic_context.doc/_module.md")]

use obscur_identity::{NodeId, ResourceId, WindowId};

#[doc = include_str!("diagnostic_context.doc/DiagnosticContext.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DiagnosticContext {
    #[doc = include_str!("diagnostic_context.doc/node.md")]
    pub node: Option<NodeId>,
    #[doc = include_str!("diagnostic_context.doc/window.md")]
    pub window: Option<WindowId>,
    #[doc = include_str!("diagnostic_context.doc/resource.md")]
    pub resource: Option<ResourceId>,
}
