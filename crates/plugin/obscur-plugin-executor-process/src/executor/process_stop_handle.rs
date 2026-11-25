use std::{
    io,
    process::ExitStatus,
    sync::{Arc, Mutex, MutexGuard},
};

use super::{child_control::ChildControl, forced_stop::ForcedStop};

/// Independent termination control for the same direct child.
#[derive(Clone)]
pub struct ProcessStopHandle {
    child: Arc<Mutex<ChildControl>>,
}

impl ProcessStopHandle {
    pub(super) fn new(child: ChildControl) -> Self {
        Self {
            child: Arc::new(Mutex::new(child)),
        }
    }

    pub(super) fn try_wait(&self) -> io::Result<Option<ExitStatus>> {
        self.child()?.try_wait()
    }

    pub(super) fn forced_stop(&self) -> io::Result<Option<ForcedStop>> {
        Ok(self.child()?.forced_stop)
    }

    pub(super) fn expire(&self) -> io::Result<ExitStatus> {
        self.stop_for(ForcedStop::Timeout)
    }

    /// Terminates and reaps the child after a host resource monitor detects a quota
    /// violation. If the child already exited, preserves its original exit reason.
    pub fn stop_for_quota(&self) -> io::Result<ExitStatus> {
        self.stop_for(ForcedStop::Quota)
    }

    fn stop_for(&self, reason: ForcedStop) -> io::Result<ExitStatus> {
        let mut child = self.child()?;
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        let status = child.stop()?;
        child.forced_stop = Some(reason);
        Ok(status)
    }

    /// Terminates and reaps the child, even if its transport worker is blocked.
    /// Calling this after exit returns the existing exit status.
    pub fn stop(&self) -> io::Result<ExitStatus> {
        self.child()?.stop()
    }

    fn child(&self) -> io::Result<MutexGuard<'_, ChildControl>> {
        self.child
            .lock()
            .map_err(|_| io::Error::other("plugin process control poisoned"))
    }
}
