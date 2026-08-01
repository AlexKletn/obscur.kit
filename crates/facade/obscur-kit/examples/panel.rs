#![doc = include_str!("panel.doc/_module.md")]
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

#[doc = include_str!("panel.doc/BAR_HEIGHT.md")]
const BAR_HEIGHT: u32 = 36;

#[doc = include_str!("panel.doc/BAR_HEIGHT_PX.md")]
const BAR_HEIGHT_PX: f32 = 36.0;

#[doc = include_str!("panel.doc/CSS.md")]
const CSS: &str = "\
.bar    { height: 100%; \
          background-color: rgba(18, 22, 33, 0.82); \
          color: #dfe6ff; font-size: 15px; \
          display: flex; flex-direction: row; align-items: center; \
          padding-left: 14px; padding-right: 14px; gap: 10px; }\n\
.title  { color: #8fa3d8; font-size: 15px; white-space: nowrap; }\n\
.spacer { flex: 1; }\n\
.count  { color: #ffffff; font-size: 15px; white-space: nowrap; }\n\
.clock  { color: #b9c6ea; font-size: 15px; white-space: nowrap; }\n\
.key    { flex: 0 0 auto; white-space: nowrap; \
          padding: 4px 12px; border: 0; border-radius: 8px; \
          background-color: rgba(38, 46, 69, 0.9); color: #dfe6ff; \
          font-size: 15px; \
          display: flex; justify-content: center; align-items: center; }\n\
.key:hover  { background-color: rgba(56, 68, 102, 0.95); }\n\
.key:active { background-color: rgba(84, 104, 160, 1.0); }\n\
";

#[doc = include_str!("panel.doc/Clock.md")]
type Clock = Arc<Mutex<String>>;

#[doc = include_str!("panel.doc/Panel.md")]
struct Panel {
    #[doc = include_str!("panel.doc/taps.md")]
    taps: u32,
    #[doc = include_str!("panel.doc/reserving.md")]
    reserving: bool,
    #[doc = include_str!("panel.doc/clock.md")]
    clock: String,
}

impl Panel {
    #[doc = include_str!("panel.doc/new.md")]
    const fn new() -> Self {
        Self {
            taps: 0,
            reserving: true,
            clock: String::new(),
        }
    }

    #[doc = include_str!("panel.doc/tap.md")]
    const fn tap(&mut self) {
        self.taps = self.taps.saturating_add(1);
    }

    #[doc = include_str!("panel.doc/reset.md")]
    const fn reset(&mut self) {
        self.taps = 0;
    }

    #[doc = include_str!("panel.doc/toggle_presentation.md")]
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

#[doc = include_str!("panel.doc/key.md")]
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

#[doc = include_str!("panel.doc/now.md")]
fn now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let day = secs % 86_400;
    format!("{:02}:{:02}:{:02}", day / 3600, (day % 3600) / 60, day % 60)
}
