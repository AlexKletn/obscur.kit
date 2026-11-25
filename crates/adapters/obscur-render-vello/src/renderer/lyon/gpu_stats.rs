//! Opt-in nonblocking GPU timing for submitted Lyon frames.

use wgpu_profiler::{GpuProfiler, GpuProfilerQuery, GpuProfilerSettings};

pub(super) struct GpuStats {
    profiler: GpuProfiler,
    sequence: u64,
    group_timing: bool,
}

impl GpuStats {
    pub(super) fn new(device: &wgpu::Device) -> Option<Self> {
        if !std::env::var_os("OBSCUR_GPU_STATS").is_some_and(|value| value == "1") {
            return None;
        }
        let required =
            wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS;
        if !device.features().contains(required) {
            eprintln!("[gpu-stats] renderer=lyon encoder timestamp queries unavailable");
            return None;
        }
        let settings = GpuProfilerSettings {
            enable_debug_groups: false,
            max_num_pending_frames: 16,
            ..Default::default()
        };
        GpuProfiler::new(device, settings)
            .map(|profiler| Self {
                profiler,
                sequence: 0,
                group_timing: std::env::var_os("OBSCUR_GPU_GROUP_STATS")
                    .is_some_and(|value| value == "1"),
            })
            .map_err(|error| eprintln!("[gpu-stats] renderer=lyon {error:?}"))
            .ok()
    }

    pub(super) fn begin(&mut self, encoder: &mut wgpu::CommandEncoder) -> GpuProfilerQuery {
        self.sequence += 1;
        self.profiler
            .begin_query(self.sequence.to_string(), encoder)
    }

    pub(super) fn group_timing_enabled(&self) -> bool {
        self.group_timing
    }

    pub(super) fn group_pass(
        scope: Option<(&Self, &GpuProfilerQuery)>,
        encoder: &mut wgpu::CommandEncoder,
        label: &'static str,
        encode: impl FnOnce(&mut wgpu::CommandEncoder),
    ) {
        if let Some((stats, frame)) = scope {
            let query = stats
                .profiler
                .begin_query(label, encoder)
                .with_parent(Some(frame));
            encode(encoder);
            stats.profiler.end_query(encoder, query);
        } else {
            encode(encoder);
        }
    }

    pub(super) fn resolve(&mut self, encoder: &mut wgpu::CommandEncoder, query: GpuProfilerQuery) {
        self.profiler.end_query(encoder, query);
        self.profiler.resolve_queries(encoder);
    }

    pub(super) fn submitted(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if let Err(error) = self.profiler.end_frame() {
            eprintln!("[gpu-stats] renderer=lyon {error:?}");
            return;
        }
        let _ = device.poll(wgpu::PollType::Poll);
        while let Some(results) = self
            .profiler
            .process_finished_frame(queue.get_timestamp_period())
        {
            for result in results {
                if let Some(time) = result.time {
                    println!(
                        "[gpu-frame] renderer=lyon submission={} raster_ms={:.3}",
                        result.label,
                        (time.end - time.start) * 1_000.0
                    );
                    let mut totals = [(0, 0.0_f64); 3];
                    for pass in result.nested_queries {
                        let index = match pass.label.as_str() {
                            "root" => 0,
                            "group" => 1,
                            "effect" => 2,
                            _ => continue,
                        };
                        if let Some(time) = pass.time {
                            totals[index].0 += 1;
                            totals[index].1 += (time.end - time.start) * 1_000.0;
                        }
                    }
                    if totals.iter().any(|(count, _)| *count != 0) {
                        let [(root, root_ms), (group, group_ms), (effect, effect_ms)] = totals;
                        println!(
                            "[gpu-lyon-groups] submission={} root_passes={root} root_ms={root_ms:.3} group_passes={group} group_ms={group_ms:.3} effect_batches={effect} effect_ms={effect_ms:.3}",
                            result.label,
                        );
                    }
                }
            }
        }
    }
}
