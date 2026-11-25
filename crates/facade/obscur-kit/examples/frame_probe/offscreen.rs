//! Run the same tile animation against a native GPU texture without a window.

use std::sync::Arc;
use std::time::{Duration, Instant};

use obscur_kit::RenderBackend;
use obscur_kit::bootstrap::{CssPipeline, KitBootstrap, StderrDiagnostics};
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::primitives::Container;
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::renderer::Renderer;
use obscur_kit::render::target::{RenderTarget, TextureTarget};
use obscur_kit::render_vello::renderer::{VelloDevice, VelloRenderer};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::frame::FrameReason;
use obscur_kit::scene::scene::SceneSnapshot;
use obscur_kit::view::view::View;

use super::{probe_stylesheet, tiles};

pub(super) fn run(
    backend: RenderBackend,
    mode: &'static str,
    args: &[String],
) -> Result<(), String> {
    let (width, height) = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--size="))
        .unwrap_or("720x480")
        .split_once('x')
        .and_then(|(w, h)| Some((w.parse::<u16>().ok()?, h.parse::<u16>().ok()?)))
        .filter(|&(w, h)| w > 0 && h > 0)
        .ok_or_else(|| "expected --size=<width>x<height> with nonzero dimensions".to_owned())?;
    let size = Size::new(f32::from(width), f32::from(height));
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter = pollster::block_on(instance.request_adapter(&Default::default()))
        .map_err(|error| error.to_string())?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_limits: adapter.limits(),
        required_features: VelloDevice::diagnostic_features(&adapter),
        ..Default::default()
    }))
    .map_err(|error| error.to_string())?;
    if u32::from(width.max(height)) > device.limits().max_texture_dimension_2d {
        return Err("requested target exceeds the GPU texture limit".into());
    }
    let mut renderer = match backend {
        RenderBackend::Vello => VelloRenderer::default().with_device(
            VelloDevice::new(device.clone(), queue).map_err(|error| error.to_string())?,
        ),
        RenderBackend::VelloHybrid => {
            VelloRenderer::default().with_hybrid_device(device.clone(), queue)
        }
        RenderBackend::Lyon => {
            let lyon = VelloDevice::lyon(device.clone(), queue);
            let lyon = if std::env::var_os("OBSCUR_LYON_STRICT").is_some_and(|value| value == "1") {
                lyon.without_lyon_fallback()
            } else {
                lyon
            };
            VelloRenderer::default().with_device(lyon)
        }
    };
    renderer.set_diagnostic_sink(Arc::new(StderrDiagnostics::new()));

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), size);
    KitBootstrap::new().install_default_stylesheet(&mut runtime);
    let css = CssSource::anonymous(probe_stylesheet(mode, args));
    CssPipeline::new().install(&mut runtime, &[css]);
    runtime.set_root_view(move || {
        View::single(
            Container::new()
                .child(tiles(mode).into_view_node().class("scene"))
                .into_view_node()
                .class("root"),
        )
    });
    runtime.request_frame(FrameReason::UiCommit);
    let mut probe = OffscreenProbe {
        runtime,
        renderer,
        device,
        target: RenderTarget::texture(TextureTarget::new(size)),
    };
    let frozen = if args.iter().any(|arg| arg == "--freeze") {
        for _ in 0..16 {
            probe.runtime.advance_motion(1.0 / 120.0);
            probe.runtime.run_due_frame_reactive();
        }
        Some(probe.runtime.snapshot())
    } else {
        None
    };
    probe.measure(mode, frozen.as_ref(), 16, 8)?;
    let gpu = adapter.get_info();
    println!(
        "[offscreen] renderer={} mode={mode} frozen={} size={width}x{height} adapter={:?} backend={:?}",
        backend.name(),
        frozen.is_some(),
        gpu.name,
        gpu.backend
    );
    for batch in [1, 8] {
        probe.measure(mode, frozen.as_ref(), 216, batch)?;
    }
    Ok(())
}

struct OffscreenProbe {
    runtime: UiRuntime,
    renderer: VelloRenderer,
    device: wgpu::Device,
    target: RenderTarget,
}

impl OffscreenProbe {
    fn measure(
        &mut self,
        mode: &str,
        frozen: Option<&SceneSnapshot>,
        frames: u32,
        batch: u32,
    ) -> Result<(), String> {
        let started = Instant::now();
        let mut prepare = Duration::ZERO;
        let mut submit = Duration::ZERO;
        let mut wait = Duration::ZERO;
        for frame in 1..=frames {
            let before_prepare = Instant::now();
            let snapshot = if let Some(snapshot) = frozen {
                snapshot.clone()
            } else if mode == "idle" && !self.runtime.has_pending_frame() {
                self.runtime.snapshot()
            } else {
                self.runtime.advance_motion(1.0 / 120.0);
                self.runtime
                    .run_due_frame_reactive()
                    .ok_or_else(|| format!("scene stopped requesting frames at sample {frame}"))?
                    .snapshot()
                    .clone()
            };
            let submission = FrameSubmission::new(RenderFrame::new(snapshot, self.target.size()));
            let before_submit = Instant::now();
            prepare += before_submit - before_prepare;
            self.renderer
                .render(&submission, &self.target)
                .map_err(|error| error.to_string())?;
            submit += before_submit.elapsed();
            if frame % batch == 0 {
                let before_wait = Instant::now();
                self.device
                    .poll(wgpu::PollType::wait_indefinitely())
                    .map_err(|error| error.to_string())?;
                wait += before_wait.elapsed();
            }
        }
        if frames > 16 {
            let count = f64::from(frames);
            let ms = |time: Duration| time.as_secs_f64() * 1000.0 / count;
            println!(
                "[offscreen] batch={batch} frames={frames} completed_fps={:.2} frame_ms={:.3} prepare_ms={:.3} submit_ms={:.3} wait_ms={:.3}",
                count / started.elapsed().as_secs_f64(),
                ms(started.elapsed()),
                ms(prepare),
                ms(submit),
                ms(wait)
            );
        }
        Ok(())
    }
}
