use std::process::ExitStatus;

/// Result of a process watchdog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessWatchdogOutcome {
    /// The host cancelled the deadline before it elapsed.
    Disarmed,
    /// The deadline elapsed. Carries the direct child's exit status.
    Expired(ExitStatus),
}
