//! Child-process lifecycle for the framed service transport.

use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};
use std::{future::Future, io};

use obscur_plugin_host::{
    service::{JsonServiceBridge, ServiceFrameCodec, ServiceSession},
    storage::{PluginStorage, StateStore, StorageNamespace, load_compatible_state},
};
use obscur_plugin_model::{lifecycle::ShutdownConcern, manifest::VersionReq};
use serde::{Serialize, de::DeserializeOwned};

use super::{ProcessShutdownChannel, ProcessWatchdog, ProcessWatchdogOutcome};
use super::{ProcessStopHandle, child_control::ChildControl, forced_stop::ForcedStop};
#[cfg(target_os = "linux")]
use super::{child_control::terminate, process_group::ProcessGroup};
use crate::transport::FramedTransport;

/// Owns a host-launched child and closes, terminates, and reaps it on stop.
pub struct ProcessExecutor {
    control: ProcessStopHandle,
    transport: FramedTransport<ChildStdout, ChildStdin>,
    shutdown_channel: Option<ProcessShutdownChannel>,
}

impl ProcessExecutor {
    /// Launches a host-approved command and connects its standard streams.
    ///
    /// The host supplies the executable or sandbox launcher, its environment,
    /// and initial handle IDs. Stdout is reserved for protocol frames. Stderr
    /// follows the command's configuration.
    pub fn spawn(
        command: &mut Command,
        codec: ServiceFrameCodec,
        session: ServiceSession,
    ) -> io::Result<Self> {
        Self::attach(ChildControl::new(spawn_piped(command)?), codec, session)
    }

    /// Loads the latest committed state for the authenticated installation before launching a child. The trusted launch callback receives the validated snapshot and chooses how to pass it to the child. No snapshot is passed when the namespace has no committed state.
    pub fn spawn_from_state(
        store: &(impl StateStore + ?Sized),
        namespace: &StorageNamespace,
        accepted: &VersionReq,
        codec: ServiceFrameCodec,
        session: ServiceSession,
        prepare: impl FnOnce(Option<&PluginStorage>) -> io::Result<Command>,
    ) -> io::Result<Self> {
        let principal = session.principal();
        if principal.plugin() != namespace.plugin()
            || principal.installation() != namespace.installation()
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "state belongs to another plugin installation",
            ));
        }
        let snapshot = load_compatible_state(store, namespace, accepted)?;
        Self::spawn(&mut prepare(snapshot.as_ref())?, codec, session)
    }

    /// Launches a plugin in a dedicated Linux process group.
    ///
    /// The executor must be the sole reaper for the child. Descendants
    /// that leave the group are outside the executor's termination scope.
    #[cfg(target_os = "linux")]
    pub fn spawn_in_process_group(
        mut command: Command,
        codec: ServiceFrameCodec,
        session: ServiceSession,
    ) -> io::Result<Self> {
        use std::os::unix::process::CommandExt;

        let mut child = spawn_piped(command.process_group(0))?;
        let group = match ProcessGroup::new(&child) {
            Ok(group) => group,
            Err(error) => {
                let _ = terminate(&mut child);
                return Err(error);
            }
        };
        Self::attach(ChildControl::new(child).with_group(group), codec, session)
    }

    fn attach(
        mut control: ChildControl,
        codec: ServiceFrameCodec,
        session: ServiceSession,
    ) -> io::Result<Self> {
        let Some((input, output)) = control.take_streams() else {
            let _ = control.stop();
            return Err(io::Error::other("plugin process streams unavailable"));
        };
        Ok(Self {
            control: ProcessStopHandle::new(control),
            transport: FramedTransport::new(codec, input, output, session),
            shutdown_channel: None,
        })
    }

    /// Attaches the authenticated one-shot shutdown channel for this child. The launch callback supplies a fresh channel for every restart.
    #[must_use]
    pub fn with_shutdown_channel(mut self, channel: ProcessShutdownChannel) -> Self {
        self.shutdown_channel = Some(channel);
        self
    }

    /// Asks the running child for an advisory concern and consumes its attached channel. Returns no concern when the child has exited or has no channel. Run on the process worker.
    pub fn shutdown_requested(&mut self, deadline: Instant) -> io::Result<Option<ShutdownConcern>> {
        if self.try_wait()?.is_some() {
            return Ok(None);
        }
        match self.shutdown_channel.take() {
            Some(channel) => channel.request(deadline),
            None => Ok(None),
        }
    }

    /// Serves one call on the blocking transport worker.
    ///
    /// EOF or a protocol error releases the session. The owner can then wait
    /// for normal exit or stop the process.
    pub async fn serve_next<Req: DeserializeOwned, Res: Serialize>(
        &mut self,
        bridge: &JsonServiceBridge<'_, Req, Res>,
    ) -> io::Result<bool> {
        self.transport.serve_next(bridge).await
    }

    /// Serves one frame with a deadline for host-side handler dispatch.
    /// The deadline factory runs after the blocking frame read.
    pub async fn serve_next_until<
        Req: DeserializeOwned,
        Res: Serialize,
        Timer: Future<Output = ()>,
    >(
        &mut self,
        bridge: &JsonServiceBridge<'_, Req, Res>,
        deadline: impl FnOnce() -> Timer,
    ) -> io::Result<bool> {
        self.transport.serve_next_until(bridge, deadline).await
    }

    /// Returns independent process control for a watchdog or supervisor thread.
    #[must_use]
    pub fn stop_handle(&self) -> ProcessStopHandle {
        self.control.clone()
    }

    /// Reports and reaps an exited process without blocking.
    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        let status = self.control.try_wait()?;
        if status.is_some() {
            self.transport.close();
            self.shutdown_channel = None;
        }
        Ok(status)
    }

    pub(super) fn forced_stop(&self) -> io::Result<Option<ForcedStop>> {
        self.control.forced_stop()
    }

    /// Closes the session and waits for normal exit. May block until the child exits.
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        self.transport.close();
        self.shutdown_channel = None;
        // Keep the child lock available to a concurrent stop request.
        loop {
            if let Some(status) = self.try_wait()? {
                return Ok(status);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Closes the transport and revokes the service session, allowing the child to
    /// exit after observing end of input. Call on the process worker. If the child
    /// outlives the deadline, the watchdog terminates and reaps it and the method
    /// returns a timed-out error.
    pub fn shutdown_until(&mut self, deadline: Instant) -> io::Result<ExitStatus> {
        let watchdog = ProcessWatchdog::arm_until(self.stop_handle(), deadline)?;
        self.transport.close();
        self.shutdown_channel = None;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                watchdog.wait()?;
                break;
            }
            if let Some(status) = self.try_wait()? {
                if watchdog.disarm()? == ProcessWatchdogOutcome::Disarmed {
                    return Ok(status);
                }
                break;
            }
            std::thread::sleep(remaining.min(Duration::from_millis(10)));
        }
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "plugin process reached its shutdown deadline",
        ))
    }

    /// Closes the session, terminates a running child, and reaps it.
    pub fn stop(&mut self) -> io::Result<ExitStatus> {
        self.transport.close();
        self.shutdown_channel = None;
        self.control.stop()
    }
}

impl Drop for ProcessExecutor {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn spawn_piped(command: &mut Command) -> io::Result<Child> {
    command.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()
}
