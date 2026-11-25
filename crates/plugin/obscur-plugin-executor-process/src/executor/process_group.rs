use std::{io, process::Child};

use rustix::{
    io::Errno,
    process::{Pid, Signal, WaitId, WaitIdOptions, kill_process_group, waitid},
};

pub(super) struct ProcessGroup(Pid);

impl ProcessGroup {
    pub(super) fn new(child: &Child) -> io::Result<Self> {
        i32::try_from(child.id())
            .ok()
            .and_then(Pid::from_raw)
            .map(Self)
            .ok_or_else(|| io::Error::other("invalid plugin process group ID"))
    }

    pub(super) fn leader_exited(&self) -> io::Result<bool> {
        loop {
            match waitid(
                WaitId::Pid(self.0),
                WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT,
            ) {
                Ok(status) => return Ok(status.is_some()),
                Err(Errno::INTR) => {}
                Err(error) => return Err(error.into()),
            }
        }
    }

    pub(super) fn stop(&self) -> io::Result<()> {
        // Check that the leader is still ours before using its numeric ID.
        self.leader_exited()?;
        match kill_process_group(self.0, Signal::KILL) {
            Ok(()) | Err(Errno::SRCH) => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}
