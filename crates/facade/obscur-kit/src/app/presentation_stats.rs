//! Sampling of frame cadence from compositor presentation feedback.

/// An approximately one-second window of compositor presentation timestamps.
#[derive(Default)]
pub(super) struct PresentationStats {
    start: Option<u64>,
    intervals: u32,
    first_frame: u64,
}

impl PresentationStats {
    /// Reset cadence and ignore callbacks from previous backends and the warm-up frame.
    pub(super) fn restart(&mut self, first_frame: u64) {
        *self = Self {
            first_frame,
            ..Self::default()
        };
    }

    /// Process compositor feedback and publish the latest completed cadence sample.
    pub(super) fn poll(
        &mut self,
        presentation: &mut obscur_wayland_effects::WindowPresentation<
            std::sync::Arc<winit::window::Window>,
        >,
        runtime: &mut crate::runtime::UiRuntime,
        trace: bool,
        mut settled: impl FnMut(u64, Option<u64>),
    ) {
        use obscur_diagnostics::{
            Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSubsystem,
        };

        let mut cadence = None;
        match presentation.poll() {
            Ok(events) => {
                for (frame, timestamp) in events {
                    settled(frame, timestamp);
                    if trace {
                        cadence = self.record(frame, timestamp).or(cadence);
                    }
                }
            }
            Err(error) => eprintln!("[window-performance] presentation feedback: {error}"),
        }
        if let Some((fps, mean)) = cadence {
            println!("[window-performance] shown_fps={fps:.2} shown_frame_ms={mean:.3}");
            runtime.report_diagnostic(&Diagnostic::new(
                DiagnosticCode::new("window.performance"),
                DiagnosticSeverity::Info,
                DiagnosticSubsystem::Render,
                format!("{fps:.1} FPS · {mean:.2} ms"),
            ));
        }
    }

    /// Record a frame outcome and return displayed FPS and mean interval in milliseconds when a sample completes.
    #[allow(clippy::cast_precision_loss)]
    fn record(&mut self, frame: u64, timestamp: Option<u64>) -> Option<(f64, f64)> {
        let Some(timestamp) = timestamp else {
            println!("[window-discarded] id={frame}");
            return None;
        };
        println!("[window-shown] id={frame} at_us={timestamp}");

        if frame < self.first_frame {
            return None;
        }
        let start = *self.start.get_or_insert(timestamp);
        let elapsed = timestamp.saturating_sub(start);
        if elapsed == 0 {
            return None;
        }
        self.intervals += 1;
        if elapsed < 1_000_000 {
            return None;
        }
        let mean = elapsed as f64 / (f64::from(self.intervals) * 1000.0);
        self.start = Some(timestamp);
        self.intervals = 0;
        Some((1000.0 / mean, mean))
    }
}

pub(super) fn unpaced_wayland_redraw() -> bool {
    static UNPACED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *UNPACED.get_or_init(|| {
        std::env::var_os("OBSCUR_WAYLAND_REDRAW_UNPACED").is_some_and(|value| value == "1")
    })
}

#[cfg(test)]
mod tests {
    use super::PresentationStats;

    /// Presentation feedback from previous backends and the warm-up frame cannot enter the next cadence sample.
    #[test]
    fn restart_ignores_late_feedback() {
        let mut stats = PresentationStats::default();
        assert_eq!(stats.record(4, Some(100_000)), None);
        stats.restart(7);
        assert_eq!(stats.record(5, Some(200_000)), None);
        assert_eq!(stats.record(6, Some(300_000)), None);
        assert_eq!(stats.record(7, Some(400_000)), None);
        assert_eq!(stats.record(8, Some(900_000)), None);
        assert_eq!(stats.record(9, Some(1_400_000)), Some((2.0, 500.0)));
    }
}
