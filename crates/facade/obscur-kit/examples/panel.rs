#![doc = include_str!("panel.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use obscur_kit::identity::InternedSymbol;
use obscur_kit::primitives::{Container, Text};
use obscur_kit::shell_layer::surface::LayerWaker;
use obscur_kit::view::view::{View, ViewNode};
use obscur_kit::widgets::button::Button;
use obscur_kit::windowing::layer::{LayerDisplay, LayerMargins, LayerReservation, LayerSettings};
use obscur_kit::{App, LayerControl};

#[doc = include_str!("panel.doc/BAR_HEIGHT.md")]
const BAR_HEIGHT: u32 = 36;

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
    fn toggle_presentation(&mut self) -> LayerSettings {
        self.reserving = !self.reserving;
        if self.reserving {
            LayerSettings::top_bar(BAR_HEIGHT)
        } else {
            // The same bar, floating: still anchored across the top, but taking
            // no space from anyone and held clear of the edge. Only the height
            // is stated — the width is the anchor's to stretch.
            LayerSettings::top_bar(BAR_HEIGHT)
                .with_reservation(LayerReservation::None)
                .with_margin(LayerMargins::uniform(12))
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

fn main() -> Result<(), obscur_kit::AppError> {
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

    // `panel eDP-1` pins the bar to that monitor; with no argument the platform
    // chooses, which on a multi-head desktop means "wherever the focus happened to
    // be" — rarely what a panel wants. The name is resolved to an id here, because
    // an id is the platform-neutral currency and a connector name is not.
    let mut settings = LayerSettings::top_bar(BAR_HEIGHT);
    if let Some(wanted) = std::env::args().nth(1) {
        let displays = App::displays()?;
        if let Some(display) = displays.iter().find(|display| display.name() == wanted) {
            settings = settings.with_display(display.id());
        } else {
            let names: Vec<&str> = displays.iter().map(LayerDisplay::name).collect();
            eprintln!("no display named {wanted:?}; this desktop has {names:?}");
        }
    }

    let app = App::new("obscur.kit panel")
        .app_id("obscur-kit-panel")
        .layer(settings);

    // How a handler reaches the live surface: the loop owns the surface, so a
    // handler leaves a request and the loop applies it on its next turn. Taken
    // from the `App` rather than made here — there is one surface, so there is
    // one handle to it.
    let float_control: LayerControl = app.layer_control();

    app.stylesheet(CSS)
        .view(move || {
            // Read the clock at build time, so the frame a wake caused shows the
            // value that caused it.
            if let Ok(text) = view_clock.lock() {
                view_state.borrow_mut().clock.clone_from(&text);
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
            let settings = float_state.borrow_mut().toggle_presentation();
            float_control.reconfigure(settings);
        })
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
