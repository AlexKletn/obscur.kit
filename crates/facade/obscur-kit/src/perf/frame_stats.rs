use std::{collections::VecDeque, time::Duration};

use super::{FrameSample, ms};

#[doc = include_str!("../../docs/api/perf.doc/FrameStats.md")]
#[derive(Debug, Clone)]
pub struct FrameStats {
    samples: VecDeque<FrameSample>,
    capacity: usize,
}

impl Default for FrameStats {
    fn default() -> Self {
        Self::new(120)
    }
}

impl FrameStats {
    /// Creates a stats window retaining the last `capacity` frames (min `1`).
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(capacity.max(1)),
            capacity: capacity.max(1),
        }
    }

    /// Records one frame's CPU and present durations, evicting the oldest sample
    /// when the window is full.
    pub fn record(&mut self, cpu: Duration, present: Duration) {
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(FrameSample { cpu, present });
    }

    /// The number of frames currently in the window.
    #[must_use]
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Whether no frames have been recorded yet.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// The mean of `f` over the window, or `Duration::ZERO` when empty.
    fn mean(&self, f: impl Fn(&FrameSample) -> Duration) -> Duration {
        let n = self.samples.len();
        if n == 0 {
            return Duration::ZERO;
        }
        let total: Duration = self.samples.iter().map(&f).sum();
        total / n as u32
    }

    /// The mean CPU (layout/compose) time per frame.
    #[must_use]
    pub fn avg_cpu(&self) -> Duration {
        self.mean(|s| s.cpu)
    }

    /// The mean present (GPU) time per frame.
    #[must_use]
    pub fn avg_present(&self) -> Duration {
        self.mean(|s| s.present)
    }

    /// The mean total time per frame.
    #[must_use]
    pub fn avg_total(&self) -> Duration {
        self.mean(|s| s.total())
    }

    /// The worst (largest) total frame time in the window.
    #[must_use]
    pub fn max_total(&self) -> Duration {
        self.samples
            .iter()
            .map(|s| s.total())
            .max()
            .unwrap_or(Duration::ZERO)
    }

    /// The sustainable FPS implied by the mean frame cost (`1000 / avg_total_ms`)
    /// — the rate the frame *could* be produced at back-to-back. `0.0` when
    /// empty. For the *actual* present rate under a continuous redraw, divide the
    /// frames recorded by the wall-clock interval instead.
    #[must_use]
    pub fn cost_fps(&self) -> f32 {
        let ms = self.avg_total().as_secs_f32() * 1000.0;
        if ms > 0.0 { 1000.0 / ms } else { 0.0 }
    }

    /// A one-line human summary: `cost fps + cpu/gpu/total ms (+ worst)`.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "~{:.0} fps · cpu {:.2}ms · gpu {:.2}ms · frame {:.2}ms (max {:.2}ms) · {} frames",
            self.cost_fps(),
            ms(self.avg_cpu()),
            ms(self.avg_present()),
            ms(self.avg_total()),
            ms(self.max_total()),
            self.len(),
        )
    }

    /// Empties the window.
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}
