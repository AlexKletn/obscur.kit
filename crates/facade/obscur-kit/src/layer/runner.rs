//! The frame loop behind [`LayerApp::run`](super::LayerApp::run).

use crate::bootstrap::{CssPipeline, KitBootstrap};
use crate::css::source::CssSource;
use crate::geometry::Size;
use crate::identity::{InternedSymbol, RuntimeId};
use crate::render::frame::{FrameSubmission, RenderFrame};
use crate::render::renderer::RendererId;
use crate::render_vello::renderer::{VelloDevice, VelloRenderer, VelloSurface};
use crate::runtime::UiRuntime;
use crate::runtime::frame::FrameReason;
use crate::shell_layer::event::LayerEvent;
use crate::shell_layer::surface::LayerWindow;
use crate::view::view::View;
use crate::windowing::layer::LayerSurfaceConfig;

use super::LayerAppError;

/// The largest time step one frame may advance motion by, matching the windowed
/// runner: a surface that was occluded or starved must not fast-forward every
/// animation by the whole gap.
const MAX_FRAME_DELTA: f32 = 0.1;

/// The frame interval used while motion is in flight, in milliseconds.
const ANIMATION_STEP_MS: u64 = 16;

/// Opens the layer surface and runs it to completion.
pub(super) fn run(
    config: LayerSurfaceConfig,
    stylesheets: Vec<String>,
    mut root: Box<dyn FnMut() -> View + 'static>,
    commands: Vec<(InternedSymbol, Box<dyn FnMut() + 'static>)>,
) -> Result<(), LayerAppError> {
    let mut window = LayerWindow::open(&config)?;

    // The protocol requires an initial commit with no buffer; the compositor
    // answers with the real size. Drawing before that is a protocol error, not a
    // race, so the first frame waits here.
    while !window.is_configured() && !window.is_closed() {
        window.dispatch()?;
    }
    if window.is_closed() {
        return Ok(());
    }

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), window.size());
    let boot = KitBootstrap::new();
    boot.install_default_stylesheet(&mut runtime);
    boot.install_image_resources(&mut runtime, ".");
    boot.install_vector_resources(&mut runtime);
    let _spawner = boot.install_blocking_executor(&mut runtime);

    let mut css = CssPipeline::new();
    let sources: Vec<CssSource> = stylesheets.into_iter().map(CssSource::anonymous).collect();
    css.install(&mut runtime, &sources);

    for (command, handler) in commands {
        runtime.on_command(command, handler);
    }
    runtime.set_root_view(move || root());

    let (mut renderer, mut surface, device, _queue) = init_gpu(&window)?;
    let fonts = boot.system_font_set().unwrap_or_default();
    let installed = boot.install_text_stack(&mut runtime, &fonts);
    for (handle, bytes) in installed.iter() {
        renderer.register_font(handle, bytes.to_vec());
    }

    // Declare the buffer scale before the first frame, or the compositor reads
    // the physical buffer as a logical size and the panel comes out at double
    // extent with everything drawn twice as large.
    #[allow(clippy::cast_possible_truncation)]
    window.set_buffer_scale(window.scale() as i32);
    runtime.set_surface(physical(window.size(), window.scale()), window.scale());
    runtime.request_frame(FrameReason::UiCommit);

    let mut last_frame: Option<std::time::Instant> = None;
    while !window.is_closed() {
        // Idle blocks in the kernel; animating polls, so a running transition is
        // not held hostage by an input-quiet compositor.
        let animating = runtime.has_pending_frame();
        let events = if animating {
            std::thread::sleep(std::time::Duration::from_millis(ANIMATION_STEP_MS));
            window.dispatch_pending()?
        } else {
            window.dispatch()?
        };

        let mut resized = false;
        for event in events {
            match event {
                LayerEvent::Configured { size } => {
                    runtime.set_surface(physical(size, window.scale()), window.scale());
                    resized = true;
                }
                LayerEvent::ScaleChanged { scale } => {
                    #[allow(clippy::cast_possible_truncation)]
                    window.set_buffer_scale(scale as i32);
                    runtime.set_surface(physical(window.size(), scale), scale);
                    resized = true;
                }
                LayerEvent::Closed => break,
                LayerEvent::Pointer(pointer) => {
                    runtime.handle_pointer(pointer);
                }
                LayerEvent::Scroll { delta_y } => {
                    // Route to whatever scroll container is under the pointer;
                    // an unconsumed notch is simply dropped, as in a window with
                    // no leftover-scroll handler.
                    let at = runtime
                        .hovered()
                        .and_then(|node| runtime.layout_boxes().get(&node).map(|b| b.bounds.origin))
                        .unwrap_or_default();
                    runtime.handle_scroll(delta_y, at);
                }
            }
        }
        if resized {
            let extent = physical(window.size(), window.scale());
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            surface.resize(&device, extent.width as u32, extent.height as u32);
            runtime.request_frame(FrameReason::WindowChange);
        }

        let now = std::time::Instant::now();
        if let Some(previous) = last_frame {
            let delta = now
                .duration_since(previous)
                .as_secs_f32()
                .min(MAX_FRAME_DELTA);
            if delta > 0.0 {
                runtime.advance_motion(delta);
            }
        }

        if runtime.has_pending_frame() {
            last_frame = Some(now);
            if runtime.run_due_frame_reactive().is_some() {
                for (handle, bytes) in installed.take_discovered() {
                    renderer.register_font(handle, bytes.to_vec());
                }
                let extent = physical(window.size(), window.scale());
                let frame = RenderFrame::new(runtime.snapshot(), extent);
                let submission = FrameSubmission::new(frame);
                if renderer.present_to_surface(&submission, &surface).is_err() {
                    surface.configure(&device);
                }
            }
        }
        window.flush()?;
    }
    Ok(())
}

/// The physical extent of a logical `size` at `scale`.
fn physical(size: Size, scale: f32) -> Size {
    Size::new(size.width * scale, size.height * scale)
}

/// Acquires a GPU device and a swapchain for the layer surface.
///
/// The same alpha handling as the windowed runner: a premultiplied surface hands
/// the root's own alpha to the compositor, so a panel is see-through exactly when
/// its stylesheet says so and there is no mode to opt into.
fn init_gpu(
    window: &LayerWindow,
) -> Result<(VelloRenderer, VelloSurface, wgpu::Device, wgpu::Queue), LayerAppError> {
    let instance = wgpu::Instance::default();
    let surface = instance
        .create_surface(window.target())
        .map_err(|error| LayerAppError::Gpu(error.to_string()))?;
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))
    .map_err(|error| LayerAppError::Gpu(error.to_string()))?;
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .map_err(|error| LayerAppError::Gpu(error.to_string()))?;

    let extent = physical(window.size(), window.scale());
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (width, height) = ((extent.width as u32).max(1), (extent.height as u32).max(1));
    let mut config = surface
        .get_default_config(&adapter, width, height)
        .ok_or_else(|| LayerAppError::Gpu("surface unsupported by adapter".to_owned()))?;
    let alpha_modes = surface.get_capabilities(&adapter).alpha_modes;
    for mode in [
        wgpu::CompositeAlphaMode::PreMultiplied,
        wgpu::CompositeAlphaMode::PostMultiplied,
    ] {
        if alpha_modes.contains(&mode) {
            config.alpha_mode = mode;
            break;
        }
    }

    let vello_device = VelloDevice::new(device.clone(), queue.clone())
        .map_err(|error| LayerAppError::Gpu(error.to_string()))?;
    let renderer = VelloRenderer::new(RendererId::from_raw(2)).with_device(vello_device);
    let vello_surface = VelloSurface::new(surface, config);
    vello_surface.configure(&device);
    Ok((renderer, vello_surface, device, queue))
}
