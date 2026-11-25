use obscur_geometry::Size;
use obscur_identity::ResourceId;
use obscur_runtime::{
    frame::{FrameReason, FrameResult},
    scheduler::BackpressureDecision,
};
use obscur_style::matching::Stylesheet;
use obscur_task::Clock;
use obscur_view::view::View;

use crate::host::HeadlessHost;
use crate::input::{DispatchOutcome, SyntheticInput};

use super::{EnvironmentChange, EventRecording, RecordedEvent, SchedulerReason, Sensitivity};

/// Runs and records operations on a headless host from the point it is attached.
pub struct HeadlessRecorder<'a> {
    host: &'a mut HeadlessHost,
    recording: EventRecording,
}

impl<'a> HeadlessRecorder<'a> {
    /// Attaches an empty recording using the default redaction policy.
    pub fn new(host: &'a mut HeadlessHost) -> Self {
        Self {
            host,
            recording: EventRecording::from_host_clock(),
        }
    }

    /// Returns the recorded operations.
    #[must_use]
    pub fn finish(self) -> EventRecording {
        self.recording
    }

    /// Dispatches input and stores it according to its sensitivity.
    pub fn input(&mut self, input: SyntheticInput, sensitivity: Sensitivity) -> DispatchOutcome {
        let outcome = self.host.dispatch_input(&input);
        self.recording
            .record_input(self.host.clock().now().as_nanos(), input, sensitivity);
        outcome
    }

    /// Advances the host clock and records the tick.
    pub fn advance_time(&mut self, nanos: u64) {
        self.host.advance_time(nanos);
        self.record(RecordedEvent::ClockTick { nanos });
    }

    /// Requests a frame and records the scheduler's actual decision.
    pub fn request_frame(&mut self, reason: FrameReason) -> BackpressureDecision {
        let decision = self.host.runtime_mut().request_frame(reason);
        self.record(RecordedEvent::FrameRequest { reason, decision });
        decision
    }

    /// Runs a frame turn and records whether a frame was produced.
    pub fn render(&mut self, view: &View, sheets: &[Stylesheet]) -> Option<FrameResult> {
        let frame = self.host.run_recorded_frame(view, sheets);
        self.record_decision(SchedulerReason::Frame, frame.is_some());
        frame
    }

    /// Runs the next due timer and records the scheduler outcome.
    pub fn run_due_timer(&mut self) -> bool {
        let accepted = self.host.run_due_timer();
        self.record_decision(SchedulerReason::Timer, accepted);
        accepted
    }

    /// Services the runtime's idle budget and records the scheduler outcome.
    pub fn service_idle_work(&mut self) -> usize {
        let units = self.host.runtime_mut().service_idle_work();
        self.record_decision(SchedulerReason::Idle, units != 0);
        units
    }

    /// Completes a pending job by ID and records a successful completion.
    pub fn complete_operation(&mut self, operation: u64) -> bool {
        let completed = self.host.complete_operation(operation);
        if completed {
            self.record(RecordedEvent::AsyncCompletion { operation });
        }
        completed
    }

    /// Records a checkpoint for a resource currently registered in the runtime.
    pub fn check_resource_version(&mut self, resource: ResourceId) -> bool {
        let Some(version) = self.host.runtime().resource_version(resource) else {
            return false;
        };
        self.record(RecordedEvent::ResourceVersion {
            resource: resource.raw(),
            version: version.stamp().raw(),
        });
        true
    }

    /// Resizes the virtual window in logical pixels and records the change.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.host
            .resize_window(Size::new(width as f32, height as f32));
        self.record(RecordedEvent::EnvironmentChange(
            EnvironmentChange::WindowResized { width, height },
        ));
    }

    /// Updates the virtual window's visibility and records the change.
    pub fn set_visible(&mut self, visible: bool) {
        self.host.set_window_visibility(visible);
        self.record(RecordedEvent::EnvironmentChange(
            EnvironmentChange::VisibilityChanged { visible },
        ));
    }

    fn record_decision(&mut self, reason: SchedulerReason, accepted: bool) {
        self.record(RecordedEvent::SchedulerDecision { reason, accepted });
    }

    fn record(&mut self, event: RecordedEvent) {
        self.recording
            .record(self.host.clock().now().as_nanos(), event);
    }
}
