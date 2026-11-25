#[doc = include_str!("../../docs/api/message/command_message.doc/ControlAction.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlAction {
    /// Pause the runtime (freeze frame production).
    Pause,
    /// Resume a paused runtime.
    Resume,
    /// Advance exactly one frame while paused.
    StepFrame,
    /// Enable (`true`) or disable (`false`) pick/inspect mode.
    SetInspectMode(bool),
}
