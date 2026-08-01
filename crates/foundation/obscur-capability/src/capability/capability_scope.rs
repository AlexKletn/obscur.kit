#![doc = include_str!("capability_scope.doc/_module.md")]

#[doc = include_str!("capability_scope.doc/CapabilityScope.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityScope {
    #[doc = include_str!("capability_scope.doc/Process.md")]
    Process,
    #[doc = include_str!("capability_scope.doc/Application.md")]
    Application,
    #[doc = include_str!("capability_scope.doc/Runtime.md")]
    Runtime,
    #[doc = include_str!("capability_scope.doc/Instance.md")]
    Instance,
}
