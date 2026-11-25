#![doc = include_str!("../../docs/api/app/gpu_instance.doc/_module.md")]

use super::AppError;

#[doc = include_str!("../../docs/api/app/gpu_instance.doc/attempts.md")]
fn attempts() -> [wgpu::Backends; 2] {
    [wgpu::Backends::PRIMARY, wgpu::Backends::all()]
}

#[doc = include_str!("../../docs/api/app/gpu_instance.doc/open.md")]
pub(super) fn open<'window>(
    make_surface: impl Fn(&wgpu::Instance) -> Result<wgpu::Surface<'window>, wgpu::CreateSurfaceError>,
) -> Result<(wgpu::Instance, wgpu::Surface<'window>, wgpu::Adapter), AppError> {
    open_with(
        attempts(),
        wgpu::InstanceDescriptor::new_without_display_handle,
        make_surface,
    )
}

/// Try the supplied graphics backends in order with a fresh instance descriptor
/// and a compatible surface for each attempt.
pub(super) fn open_with<'window>(
    backends: impl IntoIterator<Item = wgpu::Backends>,
    descriptor: impl Fn() -> wgpu::InstanceDescriptor,
    make_surface: impl Fn(&wgpu::Instance) -> Result<wgpu::Surface<'window>, wgpu::CreateSurfaceError>,
) -> Result<(wgpu::Instance, wgpu::Surface<'window>, wgpu::Adapter), AppError> {
    let mut last = None;
    for backends in backends {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            // The one field here the environment is still allowed to change,
            // and it is allowed because of what it can and cannot do: these
            // bits turn validation, debug labels and GPU-based validation on
            // and off, so the worst a wrong `WGPU_VALIDATION` produces is a
            // slower, noisier run. `backends` is not in that category — a wrong
            // value there is a program that does not start — which is why it is
            // not read from the environment at all. See `attempts`.
            flags: wgpu::InstanceFlags::from_env_or_default(),
            ..descriptor()
        });
        let surface = match make_surface(&instance) {
            Ok(surface) => surface,
            Err(error) => {
                last = Some(AppError::GpuSurface(error));
                continue;
            }
        };
        match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        })) {
            Ok(adapter) => return Ok((instance, surface, adapter)),
            Err(error) => last = Some(AppError::GpuAdapter(error)),
        }
    }
    Err(last.unwrap_or_else(|| AppError::Gpu("no GPU backend was asked for".to_owned())))
}

pub(super) fn request_device(
    adapter: &wgpu::Adapter,
) -> Result<(wgpu::Device, wgpu::Queue), AppError> {
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        // Desktop-tier defaults can reject devices whose limits the renderer supports.
        required_limits: adapter.limits(),
        required_features: obscur_render_vello::renderer::VelloDevice::diagnostic_features(adapter),
        ..Default::default()
    }))
    .map_err(AppError::GpuDevice)
}

/// Creates the renderer shared by the ordinary window and layer-shell hosts.
pub(super) fn create_renderer(
    backend: super::RenderBackend,
    id: obscur_render::renderer::RendererId,
    device: wgpu::Device,
    queue: wgpu::Queue,
) -> Result<obscur_render_vello::renderer::VelloRenderer, super::AppError> {
    let mut renderer = obscur_render_vello::renderer::VelloRenderer::new(id);
    install_backend(&mut renderer, backend, device, queue)?;
    Ok(renderer)
}

/// Construct the requested backend before replacing the one attached to the renderer.
fn install_backend(
    renderer: &mut obscur_render_vello::renderer::VelloRenderer,
    backend: super::RenderBackend,
    device: wgpu::Device,
    queue: wgpu::Queue,
) -> Result<(), super::AppError> {
    use super::RenderBackend;
    use obscur_render_vello::renderer::VelloDevice;

    RenderBackend::resolve(Some(backend))?;
    let device = match backend {
        RenderBackend::Vello => VelloDevice::new(device, queue).map_err(AppError::GpuRenderer)?,
        #[cfg(feature = "render-vello-hybrid")]
        RenderBackend::VelloHybrid => VelloDevice::hybrid(device, queue),
        #[cfg(feature = "render-lyon")]
        RenderBackend::Lyon => {
            let device = VelloDevice::lyon(device, queue);
            if std::env::var_os("OBSCUR_LYON_STRICT").is_some_and(|value| value == "1") {
                device.without_lyon_fallback()
            } else {
                device
            }
        }
        #[allow(unreachable_patterns)]
        _ => {
            return Err(super::AppError::Gpu(
                "selected renderer is unavailable in this build".into(),
            ));
        }
    };
    renderer.set_device(device);
    if backend != RenderBackend::Vello {
        renderer.set_diagnostic_sink(std::sync::Arc::new(
            crate::bootstrap::StderrDiagnostics::new(),
        ));
    }
    eprintln!("[renderer] {}", backend.name());
    Ok(())
}

/// Apply the latest backend request to the current renderer and report its outcome.
pub(super) fn apply_renderer_request(
    control: &crate::control::AppControl,
    runtime: &mut crate::runtime::UiRuntime,
    renderer: &mut obscur_render_vello::renderer::VelloRenderer,
    active: &mut super::RenderBackend,
) -> bool {
    use obscur_diagnostics::{Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSubsystem};

    let Some(backend) = control.take_renderer().filter(|backend| backend != active) else {
        return false;
    };
    let Some(gpu) = renderer.gpu_device() else {
        return false;
    };
    let (device, queue) = (gpu.device().clone(), gpu.queue().clone());
    let started = std::time::Instant::now();
    let (code, severity, message, changed) = match install_backend(renderer, backend, device, queue)
    {
        Ok(()) => {
            *active = backend;
            let ms = started.elapsed().as_secs_f64() * 1000.0;
            (
                "window.renderer.changed",
                DiagnosticSeverity::Info,
                format!("{} ({ms:.2} ms)", backend.name()),
                true,
            )
        }
        Err(error) => (
            "window.renderer.unavailable",
            DiagnosticSeverity::Warning,
            error.to_string(),
            false,
        ),
    };
    runtime.report_diagnostic(&Diagnostic::new(
        DiagnosticCode::new(code),
        severity,
        DiagnosticSubsystem::Render,
        message,
    ));
    changed
}

/// Override the window swapchain depth for frame pacing measurements.
pub(super) fn apply_frame_latency(config: &mut wgpu::SurfaceConfiguration) {
    if let Some(latency) = std::env::var("OBSCUR_FRAME_LATENCY")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| (1..=3).contains(value))
    {
        config.desired_maximum_frame_latency = latency;
        eprintln!("[surface-config] requested_frame_latency={latency}");
    }
}

#[cfg(test)]
mod tests {
    #[doc = include_str!("../../docs/api/app/gpu_instance.doc/the_last_attempt_is_everything_compiled_in.md")]
    #[test]
    fn the_last_attempt_is_everything_compiled_in() {
        // Not a GPU test: this is the plan `open` iterates, asked of `open`'s
        // own function rather than of a copy of it, and no device is opened.
        let plan = super::attempts();

        assert_eq!(
            plan.last().copied(),
            Some(wgpu::Backends::all()),
            "the last attempt must be everything compiled in: it is the one that \
             has to answer on a machine the earlier attempts have nothing for"
        );
        for (index, set) in plan.iter().enumerate() {
            assert!(
                !set.is_empty(),
                "attempt {index} asks for no backend at all — an instance that \
                 can create no surface and find no adapter"
            );
            assert!(
                !plan[..index].contains(set),
                "attempt {index} repeats an earlier one: a second full instance \
                 build and a second `request_adapter` for an answer already given"
            );
        }
        assert!(
            plan[0].contains(wgpu::Backends::VULKAN) && !plan[0].contains(wgpu::Backends::GL),
            "the first attempt is the first-tier set, and not bringing up GLES \
             beside it is the whole saving this module exists for"
        );
    }
}
