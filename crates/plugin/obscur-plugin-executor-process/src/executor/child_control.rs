use std::{
    io,
    process::{Child, ChildStdin, ChildStdout, ExitStatus},
};

use super::forced_stop::ForcedStop;
#[cfg(target_os = "linux")]
use super::process_group::ProcessGroup;

pub(super) struct ChildControl {
    child: Child,
    pub(super) forced_stop: Option<ForcedStop>,
    #[cfg(target_os = "linux")]
    group: Option<ProcessGroup>,
}

impl ChildControl {
    #[cfg(target_os = "linux")]
    pub(super) fn with_group(mut self, group: ProcessGroup) -> Self {
        self.group = Some(group);
        self
    }

    pub(super) fn take_streams(&mut self) -> Option<(ChildStdout, ChildStdin)> {
        self.child.stdout.take().zip(self.child.stdin.take())
    }

    pub(super) fn new(child: Child) -> Self {
        Self {
            child,
            forced_stop: None,
            #[cfg(target_os = "linux")]
            group: None,
        }
    }

    pub(super) fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        #[cfg(target_os = "linux")]
        if let Some(group) = &self.group {
            if !group.leader_exited()? {
                return Ok(None);
            }
            // Stop descendants before reaping the leader and releasing its ID.
            self.stop_group()?;
        }
        self.child.try_wait()
    }

    pub(super) fn stop(&mut self) -> io::Result<ExitStatus> {
        #[cfg(target_os = "linux")]
        self.stop_group()?;
        terminate(&mut self.child)
    }

    #[cfg(target_os = "linux")]
    fn stop_group(&mut self) -> io::Result<()> {
        if let Some(group) = &self.group {
            group.stop()?;
            self.group = None;
        }
        Ok(())
    }
}

pub(super) fn terminate(child: &mut Child) -> io::Result<ExitStatus> {
    if let Some(status) = child.try_wait()? {
        return Ok(status);
    }
    if let Err(error) = child.kill() {
        // A natural exit may race with the termination request.
        return child.try_wait()?.ok_or(error);
    }
    child.wait()
}
