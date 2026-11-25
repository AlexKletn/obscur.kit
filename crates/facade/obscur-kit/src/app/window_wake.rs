pub(super) enum WindowWake {
    SurfaceReady,
    #[cfg(feature = "accesskit-window")]
    Accessibility(accesskit_winit::Event),
    ViewChanged,
    AppearanceChanged,
    #[cfg(target_os = "linux")]
    WaylandPointing(Option<obscur_wayland_effects::WaylandPointingCapabilities>),
    #[cfg(all(feature = "gamepad", target_os = "linux"))]
    Gamepad(obscur_interaction::gamepad::GamepadEvent),
    #[cfg(target_os = "linux")]
    TouchpadInput(obscur_interaction::gesture::TouchpadGestureEvent),
}

#[cfg(feature = "accesskit-window")]
impl From<accesskit_winit::Event> for WindowWake {
    fn from(event: accesskit_winit::Event) -> Self {
        Self::Accessibility(event)
    }
}
