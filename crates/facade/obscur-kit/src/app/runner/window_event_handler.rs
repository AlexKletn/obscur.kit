//! Native event loop callbacks for the standalone window runner.

use std::ops::ControlFlow;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{WindowAttributes, WindowId};

#[cfg(feature = "accesskit-window")]
use super::window_accessibility::WindowAccessibility;
use super::{AppRunner, gpu_diagnostic, init_gpu, install_text, transform_component};
use crate::app::{AppError, window_wake::WindowWake};
use crate::bootstrap::KitBootstrap;
use crate::render::renderer::RendererId;
use crate::render_vello::renderer::{AcquireWake, SurfaceAcquirer, VelloSurface};
use crate::runtime::frame::FrameReason;
use crate::window_winit::conversion::KeyboardConverter;

impl ApplicationHandler<WindowWake> for AppRunner {
    /// Creates the window + GPU renderer once the platform is ready, wires the text
    /// stack, enables IME, and schedules the first frame. A window or GPU failure
    /// records the reason and exits the loop (never panics — the headless CI path).
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let mut attributes = WindowAttributes::default()
            .with_title(self.title.clone())
            .with_decorations(self.decorated)
            .with_resizable(self.resize)
            .with_content_protected(
                self.control
                    .take_window_capture_protection()
                    .unwrap_or(false),
            )
            .with_maximized(self.window_mode == crate::WindowMode::Maximized)
            .with_fullscreen(
                (self.window_mode == crate::WindowMode::Fullscreen)
                    .then_some(winit::window::Fullscreen::Borderless(None)),
            )
            .with_transparent(true)
            .with_inner_size(LogicalSize::new(
                f64::from(self.size.0),
                f64::from(self.size.1),
            ));
        if let Some((width, height)) = self.size_limits.minimum {
            attributes = attributes.with_min_inner_size(LogicalSize::new(width, height));
        }
        if let Some((width, height)) = self.size_limits.maximum {
            attributes = attributes.with_max_inner_size(LogicalSize::new(width, height));
        }
        // The Wayland `app_id` / X11 `WM_CLASS` — the stable identity a compositor
        // matches window rules on (Hyprland's `class`, sway's `app_id`) and the one
        // a taskbar keys off. winit exposes it only through the per-backend
        // extension traits, and setting both is correct: they write different
        // platform fields, so an X11 session and a Wayland one each get the id.
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            {
                use winit::platform::wayland::WindowAttributesExtWayland as _;
                attributes = attributes.with_name(self.app_id.clone(), String::new());
            }
            {
                use winit::platform::x11::WindowAttributesExtX11 as _;
                attributes = attributes.with_name(self.app_id.clone(), String::new());
            }
        }
        #[cfg(feature = "accesskit-window")]
        let attributes = attributes.with_visible(false);
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                self.init_failure = Some(AppError::WindowCreation(error));
                event_loop.exit();
                return;
            }
        };
        #[cfg(feature = "accesskit-window")]
        {
            if let Some(proxy) = self.proxy.clone() {
                self.accessibility =
                    Some(WindowAccessibility::new(&window, proxy, &mut self.runtime));
            }
            window.set_visible(true);
        }
        // Text entry starts OFF and is offered per focus by `sync_ime` below. It
        // used to be switched on here, once, for the window's whole life — which
        // reads as "IME composition works out of the box" on a desktop and as "the
        // on-screen keyboard covers half the tablet" on a tablet, in an app with
        // no field to type into.
        window.set_ime_allowed(false);

        let enabled = self
            .control
            .take_background_blur()
            .unwrap_or(self.background_blur);
        self.set_background_blur(&window, enabled);

        let (surface, config, device, queue, gpu) = match init_gpu(&window, self.prefer_opengl) {
            Ok(gpu) => gpu,
            Err(error) => {
                self.init_failure = Some(error);
                event_loop.exit();
                return;
            }
        };
        #[cfg(target_os = "linux")]
        {
            self.threaded_present =
                winit::platform::wayland::ActiveEventLoopExtWayland::is_wayland(event_loop);
        }
        self.runtime.report_diagnostic(&gpu_diagnostic(&gpu));
        let vello_surface = VelloSurface::new(surface, config);
        vello_surface.configure(&device);
        let mut renderer = match crate::app::gpu_instance::create_renderer(
            self.render_backend,
            RendererId::from_raw(1),
            device.clone(),
            queue,
        ) {
            Ok(renderer) => renderer,
            Err(error) => {
                self.init_failure = Some(error);
                event_loop.exit();
                return;
            }
        };
        self.fonts = Some(install_text(
            &KitBootstrap::new(),
            &mut self.runtime,
            &mut renderer,
        ));

        self.scale_factor = window.scale_factor();
        self.extent = (vello_surface.config().width, vello_surface.config().height);

        // The surface moves onto the acquire thread, and its wake is an
        // `EventLoopProxy`: a user event both rouses a parked loop and lands on
        // this thread, which `request_redraw` alone does not promise.
        let wake: Option<AcquireWake> = self.proxy.clone().map(|proxy| {
            Arc::new(move || {
                let _ = proxy.send_event(WindowWake::SurfaceReady);
            }) as AcquireWake
        });
        let mut acquirer = SurfaceAcquirer::spawn(vello_surface, device, wake);
        if let Some(error) = acquirer.take_startup_error() {
            self.init_failure = Some(AppError::SurfaceAcquireThread(error));
            event_loop.exit();
            return;
        }
        self.acquirer = Some(acquirer);

        self.renderer = Some(renderer);
        #[cfg(target_os = "linux")]
        {
            self.init_wayland_feedback(&window);
            use winit::platform::wayland::WindowExtWayland;
            if !self.decorated
                && let Some(toplevel) = window.xdg_toplevel()
            {
                // SAFETY: winit supplied the xdg-toplevel of the retained window.
                match unsafe {
                    obscur_wayland_effects::TouchWindowMove::new(Arc::clone(&window), toplevel)
                } {
                    Ok(touch_move) => self.wayland_touch_move = touch_move,
                    Err(error) => eprintln!("Wayland touch move unavailable: {error}"),
                }
            }
            let gesture_proxy = self.proxy.clone();
            let pointing_proxy = self.proxy.clone();
            match obscur_wayland_effects::WindowGestures::with_pointing_capabilities(
                Arc::clone(&window),
                move |event| {
                    gesture_proxy.as_ref().is_some_and(|proxy| {
                        proxy.send_event(WindowWake::TouchpadInput(event)).is_ok()
                    })
                },
                move |capabilities| {
                    pointing_proxy.as_ref().is_some_and(|proxy| {
                        proxy
                            .send_event(WindowWake::WaylandPointing(capabilities))
                            .is_ok()
                    })
                },
            ) {
                Ok(gestures) => self.wayland_gestures = gestures,
                Err(error) => eprintln!("Wayland seat and gesture reporting unavailable: {error}"),
            }
        }
        self.runtime
            .lifecycle_mut()
            .window_opened(crate::identity::WindowId::from_raw(window.id().into()));
        self.window = Some(window);
        #[cfg(all(feature = "gamepad", target_os = "linux"))]
        if let Some(proxy) = self.proxy.clone() {
            match obscur_window_winit::gamepad::GamepadSource::spawn(move |event| {
                proxy.send_event(WindowWake::Gamepad(event)).is_ok()
            }) {
                Ok(source) => self.gamepad_source = Some(source),
                Err(error) => eprintln!("Gamepad input unavailable: {error}"),
            }
        }
        self.sync_window_state();
        self.sync_surface();
        self.runtime.request_frame(FrameReason::UiCommit);
        self.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        #[cfg(feature = "accesskit-window")]
        if let (Some(accessibility), Some(window)) = (&mut self.accessibility, &self.window)
            && window.id() == window_id
        {
            accessibility.process_event(window, &event);
        }
        if matches!(
            event,
            WindowEvent::Resized(_)
                | WindowEvent::ScaleFactorChanged { .. }
                | WindowEvent::Focused(_)
                | WindowEvent::Moved(_)
                | WindowEvent::Occluded(_)
        ) {
            self.sync_window_state();
        }
        let mut primary_press = None;
        let mut touch_press = None;
        match event {
            WindowEvent::Resized(new_size) => {
                let extent = (new_size.width, new_size.height);
                if extent.0 == 0 || extent.1 == 0 {
                    return;
                }
                if self.detailed_stats {
                    println!(
                        "[window-resized] width={} height={} at_us={}",
                        extent.0,
                        extent.1,
                        self.now_micros()
                    );
                }
                self.accept_window_extent(extent);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor;
                self.sync_surface();
                self.runtime.request_frame(FrameReason::WindowChange);
                self.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => self.handle_mouse_move(position),
            WindowEvent::CursorEntered { .. } => {
                self.entry_pending = true;
            }
            WindowEvent::CursorLeft { .. } => self.handle_mouse_leave(),
            WindowEvent::MouseInput { state, button, .. } => {
                match self.handle_mouse_button(button, state) {
                    ControlFlow::Continue(press) => primary_press = press,
                    ControlFlow::Break(()) => return,
                }
            }
            WindowEvent::Touch(contact) => {
                #[cfg(target_os = "linux")]
                if winit::platform::wayland::ActiveEventLoopExtWayland::is_wayland(event_loop)
                    && self.handle_touch_window_resize(contact)
                {
                    return;
                }
                let event = self.handle_touch_contact(contact);
                if event.kind() == crate::interaction::pointer::PointerEventKind::Down {
                    touch_press = Some((contact.id, event));
                }
            }
            WindowEvent::PinchGesture { delta, phase, .. } if cfg!(target_os = "macos") => {
                self.native_touchpad_transform(
                    transform_component::TransformComponent::Magnification(delta as f32),
                    phase,
                );
            }
            WindowEvent::RotationGesture { delta, phase, .. } if cfg!(target_os = "macos") => {
                self.native_touchpad_transform(
                    transform_component::TransformComponent::Rotation(-delta.to_radians()),
                    phase,
                );
            }
            WindowEvent::MouseWheel { delta, phase, .. } => self.handle_mouse_wheel(delta, phase),
            WindowEvent::ModifiersChanged(state) => {
                // The platform's own answer, which beats the runtime's folded
                // one: a modifier released while this window was unfocused
                // produces no key event here, so the fold would keep believing
                // it is held. Alt-tab away, let go of Alt, come back.
                self.runtime
                    .set_modifiers(KeyboardConverter::modifiers(state.state()));
            }
            WindowEvent::KeyboardInput { event, .. } => self.handle_native_key(event),
            WindowEvent::Ime(ime) => self.handle_native_ime(ime),
            WindowEvent::RedrawRequested => {
                if self.detailed_stats && self.frame_due() {
                    println!(
                        "[window-redraw] next_id={} at_us={}",
                        self.frame_sequence + 1,
                        self.now_micros()
                    );
                }
                self.render_frame();
            }
            WindowEvent::Focused(false) => {
                self.runtime.cancel_indirect_gestures();
                self.request_redraw();
            }
            WindowEvent::CloseRequested => {
                self.request_window_close(event_loop, window_id);
                return;
            }
            WindowEvent::Destroyed => {
                event_loop.exit();
                return;
            }
            _ => {}
        }
        if !self.apply_window_actions(event_loop, primary_press, touch_press) {
            self.sync_ime();
        }
    }

    /// Handle a view change or a wake from the surface worker. Process a completed
    /// presentation before requesting another image. On Wayland, submit a resize
    /// frame when its matching buffer arrives.
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, wake: WindowWake) {
        #[cfg(feature = "accesskit-window")]
        if let WindowWake::Accessibility(event) = wake {
            if self
                .window
                .as_ref()
                .is_some_and(|window| window.id() == event.window_id)
                && let Some(accessibility) = &mut self.accessibility
            {
                accessibility.handle_event(event.window_event);
            }
            return;
        }
        #[cfg(all(feature = "gamepad", target_os = "linux"))]
        if let WindowWake::Gamepad(event) = wake {
            if event.is_connection_change()
                || matches!(
                    event.kind(),
                    obscur_interaction::gamepad::GamepadEventKind::ButtonUp(_)
                )
                || self
                    .window
                    .as_ref()
                    .is_some_and(|window| window.has_focus())
            {
                let now = self.now_micros();
                self.runtime.handle_gamepad(event.with_timestamp(now));
                if self.runtime.has_pending_frame() {
                    self.request_redraw();
                }
            }
            return;
        }

        #[cfg(target_os = "linux")]
        if let WindowWake::TouchpadInput(mut event) = wake {
            event.position.x *= self.scale();
            event.position.y *= self.scale();
            event.timestamp_micros = self.now_micros();
            if self.runtime.handle_touchpad_gesture(event) {
                self.request_redraw();
            }
            return;
        }
        #[cfg(target_os = "linux")]
        if let WindowWake::WaylandPointing(capabilities) = wake {
            self.runtime.set_detected_media_input(
                capabilities.map(crate::app::wayland_media_input::media_input_from_seats),
            );
            if self.runtime.has_pending_frame() {
                self.request_redraw();
            }
            return;
        }
        if matches!(wake, WindowWake::SurfaceReady)
            && let Some(committed) = self
                .acquirer
                .as_mut()
                .and_then(SurfaceAcquirer::take_presented_at)
            && let Some(commit) = self.pending_commit.take()
        {
            self.finish_commit(commit, committed, true);
        }
        if matches!(wake, WindowWake::AppearanceChanged) {
            if let Some(appearance) = &mut self.appearance {
                appearance.refresh(&mut self.runtime);
            }
            if self.runtime.has_pending_frame() {
                self.request_redraw();
            }
            return;
        }
        if matches!(wake, WindowWake::ViewChanged) {
            self.runtime.request_frame(FrameReason::UiCommit);
        }
        #[cfg(target_os = "linux")]
        if matches!(wake, WindowWake::SurfaceReady)
            && self.resize_frame_pending
            && winit::platform::wayland::ActiveEventLoopExtWayland::is_wayland(_event_loop)
        {
            // The compositor already gave us a new size. Present the matching
            // buffer as soon as it is ready, without waiting for the redraw
            // callback scheduled by the preceding frame.
            self.render_frame();
            return;
        }
        self.request_redraw();
    }

    /// Polls for landed resource loads each turn: the real blocking executor lands
    /// a `url()` image a frame later, so an otherwise idle UI repaints when it
    /// arrives (spec §1.5 / §3.3).
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        #[cfg(target_os = "linux")]
        if let Some(mover) = self.wayland_touch_move.as_mut()
            && let Err(error) = mover.refresh()
        {
            eprintln!("Wayland touch tracking failed: {error}");
            self.wayland_touch_move = None;
        }
        if let Some(window) = &self.window
            && let Some(protected) = self.control.take_window_capture_protection()
        {
            window.set_content_protected(protected);
        }
        #[cfg(target_os = "linux")]
        if winit::platform::wayland::ActiveEventLoopExtWayland::is_wayland(event_loop) {
            // Wayland can update the toplevel state without changing its size.
            self.sync_window_state();
        }
        #[cfg(target_os = "linux")]
        if self.pending_commit.is_none() {
            self.poll_wayland_presentation();
        }
        if self.apply_window_actions(event_loop, None, None) {
            return;
        }
        #[cfg(feature = "accesskit-window")]
        if let Some(accessibility) = &mut self.accessibility {
            accessibility.sync();
        }
        if self.runtime.poll_accessibility_actions() > 0 {
            self.request_redraw();
        }
        if self.runtime.poll_resource_tasks() > 0 {
            self.runtime.request_frame(FrameReason::UiCommit);
            self.request_redraw();
        }
        // A request queued from outside the input path — a stylesheet replaced when a
        // timer fired, or a background task's result — has to reach the runtime even
        // though no event arrived, and `apply` runs inside a frame. So an idle loop
        // owes a turn exactly when something is queued. Deliberately not "when a
        // registration is still unresolved": a name that matches nothing never
        // resolves, and waking for that would spin forever over one typo.
        if self.control.has_pending() {
            self.runtime.request_frame(FrameReason::UiCommit);
            self.request_redraw();
        }
        #[cfg(target_os = "linux")]
        if self.pending_commit.is_none()
            && !self.resize_frame_pending
            && let Some(extent) = self.queued_resize.take()
        {
            // A stream of configures must not invalidate every buffer before it
            // can be presented. Finish the selected size, then take the newest.
            self.apply_resize(extent);
        }
        // At the end of the event batch the worker has the latest resize.
        // Start it even if the compositor has not released the next redraw.
        if self.pending_commit.is_none() && std::mem::take(&mut self.resize_acquire_pending) {
            if let Some(acquirer) = self.acquirer.as_mut() {
                acquirer.request();
            }
            #[cfg(target_os = "linux")]
            if self.resize_frame_pending
                && winit::platform::wayland::ActiveEventLoopExtWayland::is_wayland(event_loop)
            {
                self.prepared_frame = self.prepare_frame();
            }
        }
        if self.pending_commit.is_none() {
            self.prepare_waiting_frame();
        }
        #[cfg(target_os = "linux")]
        self.schedule_wayland_redraw(event_loop);
    }
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.control.update_window_state(None);
    }
}
