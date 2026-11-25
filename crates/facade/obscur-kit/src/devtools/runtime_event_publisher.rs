use obscur_devtools_protocol::message::{EventKind, Topic};
use obscur_devtools_server::transport::EventQueue;

use crate::identity::{NodeId, VersionStamp};
use crate::runtime::UiRuntime;

#[doc = include_str!("../../docs/api/devtools.doc/RuntimeEventPublisher.md")]
pub struct RuntimeEventPublisher {
    events: EventQueue,
    /// The last commit stamp published, seeded from the runtime at
    /// [`attach`](Self::attach) so the publisher reports what happens *after* it
    /// is installed and never invents a frame for state that predates it.
    last_commit: VersionStamp,
    /// The last focus target published, seeded the same way.
    last_focus: Option<NodeId>,
}

impl RuntimeEventPublisher {
    /// Starts publishing `runtime`'s transitions onto `events`, taking the
    /// runtime's current commit version and focus as the baseline.
    #[must_use]
    pub fn attach(runtime: &UiRuntime, events: EventQueue) -> Self {
        Self {
            events,
            last_commit: runtime.commit_version(),
            last_focus: runtime.focused(),
        }
    }

    /// Publishes an event for each of `runtime`'s transitions since the last
    /// call. Cheap enough for every frame: two reads and two comparisons when
    /// nothing changed.
    pub fn publish(&mut self, runtime: &UiRuntime) {
        let commit = runtime.commit_version();
        if commit != self.last_commit {
            self.last_commit = commit;
            // See `DiagnosticEventSink::emit` on the dropped return.
            let _ = self.events.push(
                Topic::Frames,
                EventKind::FrameCommitted { id: commit.raw() },
            );
        }
        let focus = runtime.focused();
        if focus != self.last_focus {
            self.last_focus = focus;
            let _ = self.events.push(Topic::Focus, EventKind::FocusChanged);
        }
    }
}
