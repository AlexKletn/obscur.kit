//! Results of polling a supervised plugin process.

use std::{process::ExitStatus, time::Duration};

/// Describes the process or restart state observed on one poll.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSupervisorEvent {
    /// The current process is still running.
    Running,
    /// The current process exited and was reaped. A failed status schedules recovery.
    Exited(ExitStatus),
    /// The watchdog stopped the current process at its deadline. Recovery is pending.
    TimedOut(ExitStatus),
    /// The host stopped the process for exceeding a resource quota. Recovery is pending.
    QuotaExceeded(ExitStatus),
    /// Recovery is waiting for the given delay to elapse.
    RestartDeferred(Duration),
    /// The failure limit was reached. An explicit host reset is required.
    RestartBlocked,
    /// The host launched a replacement process.
    Restarted,
    /// No process is running and no restart is pending.
    Stopped,
}
