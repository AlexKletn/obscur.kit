//! Host-driven supervision of a process plugin.

use std::{io, process::ExitStatus, time::Instant};

use obscur_plugin_host::lifecycle::CrashRecovery;
use obscur_plugin_model::lifecycle::ShutdownConcern;

use super::forced_stop::ForcedStop;
use super::{ProcessExecutor, ProcessSupervisorEvent};

/// Supervises one process plugin and applies its crash recovery policy across restarts.
pub struct ProcessSupervisor {
    executor: Option<ProcessExecutor>,
    restart_pending: bool,
    recovery: CrashRecovery,
}

impl ProcessSupervisor {
    /// Starts supervising an already launched executor.
    #[must_use]
    pub fn new(executor: ProcessExecutor, recovery: CrashRecovery) -> Self {
        Self {
            executor: Some(executor),
            restart_pending: false,
            recovery,
        }
    }

    /// Borrows the current executor for transport operations. Use the supervisor shutdown method for intentional termination.
    pub fn executor_mut(&mut self) -> Option<&mut ProcessExecutor> {
        self.executor.as_mut()
    }

    /// Clears recorded failures after stable operation or an explicit host reset.
    pub fn reset_recovery(&mut self) {
        self.recovery.reset();
    }

    /// Asks the current process for an advisory shutdown concern. This does not change the restart policy or stop the child.
    pub fn shutdown_requested(&mut self, deadline: Instant) -> io::Result<Option<ShutdownConcern>> {
        match self.executor.as_mut() {
            Some(executor) => executor.shutdown_requested(deadline),
            None => Ok(None),
        }
    }

    /// Checks for exit or attempts one restart if the policy permits it. Call on the host process worker. The launch callback can restore committed state with ProcessExecutor::spawn_from_state. If launch fails, its error is returned and counted against the retry policy.
    pub fn poll(
        &mut self,
        launch: impl FnOnce() -> io::Result<ProcessExecutor>,
    ) -> io::Result<ProcessSupervisorEvent> {
        if let Some(executor) = self.executor.as_mut() {
            let Some(status) = executor.try_wait()? else {
                return Ok(ProcessSupervisorEvent::Running);
            };
            let forced_stop = executor.forced_stop()?;
            self.executor = None;
            self.restart_pending = forced_stop.is_some() || !status.success();
            if self.restart_pending {
                self.recovery.record_failure(Instant::now());
            }
            return Ok(match forced_stop {
                Some(ForcedStop::Timeout) => ProcessSupervisorEvent::TimedOut(status),
                Some(ForcedStop::Quota) => ProcessSupervisorEvent::QuotaExceeded(status),
                None => ProcessSupervisorEvent::Exited(status),
            });
        }
        if !self.restart_pending {
            return Ok(ProcessSupervisorEvent::Stopped);
        }
        let Some(delay) = self.recovery.retry_after(Instant::now()) else {
            return Ok(ProcessSupervisorEvent::RestartBlocked);
        };
        if !delay.is_zero() {
            return Ok(ProcessSupervisorEvent::RestartDeferred(delay));
        }
        self.executor = Some(launch().inspect_err(|_| {
            self.recovery.record_failure(Instant::now());
        })?);
        self.restart_pending = false;
        Ok(ProcessSupervisorEvent::Restarted)
    }

    /// Cancels any pending restart and shuts down the current executor by the deadline.
    pub fn shutdown_until(&mut self, deadline: Instant) -> io::Result<Option<ExitStatus>> {
        self.restart_pending = false;
        self.executor
            .take()
            .map(|mut executor| executor.shutdown_until(deadline))
            .transpose()
    }
}
