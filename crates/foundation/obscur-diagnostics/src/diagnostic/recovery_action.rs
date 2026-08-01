#![doc = include_str!("recovery_action.doc/_module.md")]

#[doc = include_str!("recovery_action.doc/RecoveryAction.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    #[doc = include_str!("recovery_action.doc/None.md")]
    None,
    #[doc = include_str!("recovery_action.doc/Fallback.md")]
    Fallback,
    #[doc = include_str!("recovery_action.doc/Retried.md")]
    Retried,
    #[doc = include_str!("recovery_action.doc/BoundaryReplaced.md")]
    BoundaryReplaced,
}
