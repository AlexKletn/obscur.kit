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
//! cargo run -p obscur-kit --features full --example panel
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

use obscur_kit::identity::InternedSymbol;
use obscur_kit::layer::LayerApp;
use obscur_kit::primitives::{Container, Text};
use obscur_kit::view::view::{View, ViewNode};
use obscur_kit::widgets::button::Button;

/// The bar's height in logical pixels. It is also what the bar reserves, which is
/// what makes it a panel rather than a strip windows slide beneath.
const BAR_HEIGHT: u32 = 36;

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
.key    { flex: 0 0 auto; width: 88px; white-space: nowrap; \
          padding: 4px 12px; border: 0; border-radius: 8px; \
          background-color: rgba(38, 46, 69, 0.9); color: #dfe6ff; \
          font-size: 15px; \
          display: flex; justify-content: center; align-items: center; }\n\
.key:hover  { background-color: rgba(56, 68, 102, 0.95); }\n\
.key:active { background-color: rgba(84, 104, 160, 1.0); }\n\
";

/// The panel's whole state.
struct Panel {
    /// How many times a button has been tapped.
    taps: u32,
}

impl Panel {
    /// A panel that has seen nothing yet.
    const fn new() -> Self {
        Self { taps: 0 }
    }

    /// Counts one tap.
    const fn tap(&mut self) {
        self.taps = self.taps.saturating_add(1);
    }

    /// Forgets every tap.
    const fn reset(&mut self) {
        self.taps = 0;
    }
}

fn view(state: &Panel) -> View {
    let bar = Container::new()
        .child(Text::new("obscur.kit").into_view_node().class("title"))
        .child(Container::new().into_view_node().class("spacer"))
        .child(
            Text::new(format!("taps: {}", state.taps))
                .into_view_node()
                .class("count"),
        )
        .child(key("tap", "Tap me"))
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

    let view_state = state.clone();
    let tap_state = state.clone();
    let reset_state = state.clone();

    LayerApp::top_bar("obscur-kit-panel", BAR_HEIGHT)
        .stylesheet(CSS)
        .view(move || view(&view_state.borrow()))
        .on_command(InternedSymbol::named("tap"), move || {
            tap_state.borrow_mut().tap();
        })
        .on_command(InternedSymbol::named("reset"), move || {
            reset_state.borrow_mut().reset();
        })
        .run()
}
