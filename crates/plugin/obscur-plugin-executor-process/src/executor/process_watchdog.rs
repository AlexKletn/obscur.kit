use std::{
    io,
    sync::mpsc::{self, RecvTimeoutError, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use super::ProcessStopHandle;

use super::ProcessWatchdogOutcome;

/// One-shot deadline for a direct child process.
#[must_use = "dropping the watchdog disarms it"]
pub struct ProcessWatchdog {
    cancel: Sender<Instant>,
    worker: Option<JoinHandle<io::Result<ProcessWatchdogOutcome>>>,
}

impl ProcessWatchdog {
    /// Arms a deadline starting when this method is called.
    pub fn arm(control: ProcessStopHandle, timeout: Duration) -> io::Result<Self> {
        let deadline = Instant::now().checked_add(timeout).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "process deadline out of range")
        })?;
        Self::arm_until(control, deadline)
    }

    pub(super) fn arm_until(control: ProcessStopHandle, deadline: Instant) -> io::Result<Self> {
        let (cancel, receiver) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("obscur-plugin-watchdog".into())
            .spawn(move || {
                match receiver.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                    Ok(cancelled) if cancelled < deadline => Ok(ProcessWatchdogOutcome::Disarmed),
                    Err(RecvTimeoutError::Disconnected) => Ok(ProcessWatchdogOutcome::Disarmed),
                    _ => control.expire().map(ProcessWatchdogOutcome::Expired),
                }
            })?;
        Ok(Self {
            cancel,
            worker: Some(worker),
        })
    }

    /// Cancels the timer if still within the deadline and waits for its outcome.
    pub fn disarm(mut self) -> io::Result<ProcessWatchdogOutcome> {
        self.cancel_and_join()
    }

    /// Waits for the deadline to expire and reports the termination outcome.
    pub fn wait(mut self) -> io::Result<ProcessWatchdogOutcome> {
        self.join()
    }

    fn cancel_and_join(&mut self) -> io::Result<ProcessWatchdogOutcome> {
        let _ = self.cancel.send(Instant::now());
        self.join()
    }

    fn join(&mut self) -> io::Result<ProcessWatchdogOutcome> {
        match self.worker.take() {
            Some(worker) => worker
                .join()
                .map_err(|_| io::Error::other("plugin watchdog panicked"))?,
            None => Ok(ProcessWatchdogOutcome::Disarmed),
        }
    }
}

impl Drop for ProcessWatchdog {
    fn drop(&mut self) {
        if self.worker.is_some() {
            let _ = self.cancel_and_join();
        }
    }
}
