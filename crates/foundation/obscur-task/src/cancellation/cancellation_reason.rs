#![doc = include_str!("cancellation_reason.doc/_module.md")]

#[doc = include_str!("cancellation_reason.doc/CancellationReason.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancellationReason {
    #[doc = include_str!("cancellation_reason.doc/ScopeEnded.md")]
    ScopeEnded,
    #[doc = include_str!("cancellation_reason.doc/Superseded.md")]
    Superseded,
    #[doc = include_str!("cancellation_reason.doc/Shutdown.md")]
    Shutdown,
}
