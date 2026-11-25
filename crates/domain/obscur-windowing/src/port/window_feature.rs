#[doc = include_str!("../../docs/api/port/window_backend.doc/WindowFeature.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowFeature {
    /// Window/framebuffer transparency (spec §6.2 dynamic properties).
    Transparency,
    /// Keeping a window above others (§6.2 `always_on_top`).
    AlwaysOnTop,
    /// Client-side / custom chrome with app-defined regions (§6.4).
    CustomChrome,
    /// Blur-behind / acrylic backdrop effects (§6.2, §6.4 presentation).
    BlurBehind,
    /// Exclusive (mode-setting) fullscreen (§6.5).
    ExclusiveFullscreen,
    /// Kiosk / locked presentation that suppresses system UI (§6.5).
    Kiosk,
    /// Inhibiting system sleep / screensaver (§6.5).
    SleepInhibition,
    /// Global logical display coordinates — the Wayland-honesty capability:
    /// platforms that hide global screen coordinates report this unavailable
    /// rather than fabricating an origin (§6.6).
    GlobalDisplayCoordinates,
    /// Popping the platform system/window menu (§6.4).
    SystemMenu,
    /// Per-monitor DPI / scale-factor awareness (§6.6).
    PerMonitorDpi,
}
