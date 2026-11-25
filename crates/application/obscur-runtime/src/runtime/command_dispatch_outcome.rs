#[doc = include_str!("../../docs/api/runtime/command_dispatch.doc/CommandDispatchOutcome.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CommandDispatchOutcome {
    /// Whether a §2.6 link claimed the command — the value the public dispatch
    /// entry points return.
    pub(crate) claimed: bool,
    /// Whether *anything* ran, listener or handler, so the caller schedules the
    /// frame that shows what it may have changed.
    pub(crate) anything_ran: bool,
    #[doc = include_str!("../../docs/api/runtime/command_dispatch.doc/default_prevented.md")]
    pub(crate) default_prevented: bool,
}
