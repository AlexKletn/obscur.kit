//! **A desktop panel** — a `wlr-layer-shell` bar across the top of the screen
//! (spec §6.5).
//!
//! This is the same toolkit as every other example: the same `View`, the same
//! CSS, the same runtime, the same touch handling. The only thing that differs
//! is how the surface reaches the screen — anchored to three edges of the output
//! instead of floating as an application window, and **reserving its own
//! height**, so a maximised window stops at the bar rather than sliding under it.
//!
//! ```sh
//! cargo run -p obscur-kit --features full --example panel          # compositor picks the output
//! cargo run -p obscur-kit --features full --example panel -- eDP-1 # pinned to a connector
//! ```
//!
//! Needs a compositor that implements `wlr-layer-shell` — Sway, Hyprland, river,
//! Wayfire, KDE. On GNOME the run ends with a stated capability error rather than
//! a crash, which is the point of returning one.
//!
//! # What to try
//!
//! Tap or click the buttons: they carry `:hover` and `:active`, and the tap
//! counter is ordinary application state re-rendered through the same reactive
//! loop a window uses. The bar is translucent, so whatever the compositor does
//! with the wallpaper behind it shows through.
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use obscur_kit::geometry::Size;
use obscur_kit::identity::InternedSymbol;
use obscur_kit::layer::{LayerApp, LayerHandle, LayerWaker};
use obscur_kit::primitives::{Container, Text};
use obscur_kit::view::view::{View, ViewNode};
use obscur_kit::widgets::button::Button;
use obscur_kit::windowing::layer::{
    LayerAnchor, LayerExclusiveZone, LayerOutput, LayerSurfaceConfig, Margins,
};

/// The bar's height in logical pixels. It is also what the bar reserves, which is
/// what makes it a panel rather than a strip windows slide beneath.
const BAR_HEIGHT: u32 = 36;

/// The same height as a float, for the places a `Size` is wanted.
const BAR_HEIGHT_PX: f32 = 36.0;

/// The styling. `100%` height rather than a fixed one, because the compositor —
/// not this sheet — decides the surface's real extent.
///
/// The buttons carry an explicit `width` they should not need. `flex: 0 0 auto`
/// ought to size them to their label, and in this kit it does not: an auto-basis
/// item with no `flex-grow` absorbs the row's free space and starves the items
/// that asked to grow. Measured on a 800px row — a plain text item took 700px
/// while a sibling `flex: 1` got 0. Remove the widths once that is fixed.
const CSS: &str = "\
.bar    { width: 100%; height: 100%; \
          background-color: rgba(18, 22, 33, 0.82); \
          color: #dfe6ff; font-size: 15px; \
          display: flex; flex-direction: row; align-items: center; \
          padding-left: 14px; padding-right: 14px; gap: 10px; }\n\
.title  { color: #8fa3d8; font-size: 15px; white-space: nowrap; }\n\
.spacer { flex: 1; }\n\
.count  { color: #ffffff; font-size: 15px; white-space: nowrap; }\n\
.clock  { color: #b9c6ea; font-size: 15px; white-space: nowrap; }\n\
.key    { flex: 0 0 auto; width: 88px; white-space: nowrap; \
          padding: 4px 12px; border: 0; border-radius: 8px; \
          background-color: rgba(38, 46, 69, 0.9); color: #dfe6ff; \
          font-size: 15px; \
          display: flex; justify-content: center; align-items: center; }\n\
.key:hover  { background-color: rgba(56, 68, 102, 0.95); }\n\
.key:active { background-color: rgba(84, 104, 160, 1.0); }\n\
";

/// The clock text a background thread keeps current, and the panel reads.
///
/// This is the point of the waker: nothing the compositor does makes a clock
/// tick, so without a way to reach the loop from outside it, the time would only
/// ever change when something else happened to wake it.
type Clock = Arc<Mutex<String>>;

/// The panel's whole state.
struct Panel {
    /// How many times a button has been tapped.
    taps: u32,
    /// Whether the bar currently reserves screen space, or floats over it.
    reserving: bool,
    /// The clock, as of the last time the loop was woken.
    clock: String,
}

impl Panel {
    /// A panel that has seen nothing yet.
    const fn new() -> Self {
        Self {
            taps: 0,
            reserving: true,
            clock: String::new(),
        }
    }

    /// Counts one tap.
    const fn tap(&mut self) {
        self.taps = self.taps.saturating_add(1);
    }

    /// Forgets every tap.
    const fn reset(&mut self) {
        self.taps = 0;
    }

    /// Flips between reserving screen space and floating over it, and returns
    /// the surface configuration that now applies.
    ///
    /// This is a live reconfiguration: no surface is destroyed and none is
    /// created. Layer-shell's placement properties — band, anchor, exclusive
    /// zone, margin, size — are all requests on the existing surface.
    fn toggle_presentation(&mut self, namespace: &str) -> LayerSurfaceConfig {
        self.reserving = !self.reserving;
        if self.reserving {
            LayerSurfaceConfig::top_bar(namespace, BAR_HEIGHT)
        } else {
            LayerSurfaceConfig::new(namespace)
                .with_anchor(LayerAnchor::TOP_BAR)
                .with_size(obscur_kit::geometry::Size::new(
                    0.0,
                    f32::from(u16::try_from(BAR_HEIGHT).unwrap_or(36)),
                ))
                .with_exclusive_zone(LayerExclusiveZone::None)
                .with_margin(Margins::uniform(12))
        }
    }
}

fn view(state: &Panel) -> View {
    let bar = Container::new()
        .child(Text::new("obscur.kit").into_view_node().class("title"))
        .child(Container::new().into_view_node().class("spacer"))
        .child(
            Text::new(state.clock.clone())
                .into_view_node()
                .class("clock"),
        )
        .child(
            Text::new(format!("taps: {}", state.taps))
                .into_view_node()
                .class("count"),
        )
        .child(key("tap", "Tap me"))
        .child(key(
            "float",
            if state.reserving { "Float" } else { "Reserve" },
        ))
        .child(key("reset", "Reset"))
        .into_view_node()
        .class("bar");
    View::single(bar)
}

/// One bar button, bound to `command`.
fn key(command: &str, label: &str) -> ViewNode {
    Button::new(label, InternedSymbol::named(command))
        .into_view_node()
        .class("key")
}

fn main() -> Result<(), obscur_kit::layer::LayerAppError> {
    let state = Rc::new(RefCell::new(Panel::new()));

    // A clock, and the thread that keeps it current. The thread never touches the
    // interface — it writes a string and rings the waker, and the loop rebuilds
    // the view from whatever it finds. That separation is the whole reason the
    // waker exists: a worker needs no access to the UI, only a way to say
    // "something changed".
    let clock: Clock = Arc::new(Mutex::new(now()));
    let waker = LayerWaker::new().expect("a waker");
    {
        let clock = Arc::clone(&clock);
        let waker = waker.clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                if let Ok(mut text) = clock.lock() {
                    *text = now();
                }
                waker.wake();
            }
        });
    }
    let view_clock = Arc::clone(&clock);

    let view_state = state.clone();
    let tap_state = state.clone();
    let reset_state = state.clone();
    let float_state = state.clone();

    // The handle is how a handler reaches the live surface: the loop owns it, so
    // a handler leaves a request and the loop applies it on its next turn.
    let handle = LayerHandle::new();
    let float_handle = handle.clone();

    // `panel eDP-1` pins the bar to that connector; with no argument the
    // compositor chooses, which on a multi-head desktop means "wherever the
    // focus happens to be" — rarely what a panel wants.
    let output = std::env::args()
        .nth(1)
        .map_or(LayerOutput::Compositor, |name| LayerOutput::Named(name));

    LayerApp::top_bar("obscur-kit-panel", BAR_HEIGHT)
        .output(output)
        .stylesheet(CSS)
        .view(move || {
            // Read the clock at build time, so the frame a wake caused shows the
            // value that caused it.
            if let Ok(text) = view_clock.lock() {
                view_state.borrow_mut().clock = text.clone();
            }
            view(&view_state.borrow())
        })
        .on_command(InternedSymbol::named("tap"), move || {
            tap_state.borrow_mut().tap();
        })
        .on_command(InternedSymbol::named("reset"), move || {
            reset_state.borrow_mut().reset();
        })
        .on_command(InternedSymbol::named("float"), move || {
            let config = float_state
                .borrow_mut()
                .toggle_presentation("obscur-kit-panel");
            float_handle.reconfigure(config);
        })
        .handle(handle)
        .waker(waker)
        .run()
}

/// The wall clock as `HH:MM:SS`, without pulling in a date library: seconds since
/// the epoch, folded into a day.
fn now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let day = secs % 86_400;
    format!("{:02}:{:02}:{:02}", day / 3600, (day % 3600) / 60, day % 60)
}
