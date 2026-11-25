#![doc = include_str!("frame_probe.doc/_module.md")]

#[path = "frame_probe/offscreen.rs"]
mod offscreen;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use obscur_kit::diagnostics::{Diagnostic, DiagnosticSink};
use obscur_kit::identity::InternedSymbol;
use obscur_kit::primitives::{Container, Text};
use obscur_kit::view::view::View;
use obscur_kit::widgets::button::Button;
use obscur_kit::{App, AppControl, RenderBackend};

const CHANGE_MODE: InternedSymbol = InternedSymbol::named("change-mode");
const CHANGE_RENDERER: InternedSymbol = InternedSymbol::named("change-renderer");
const TOGGLE_BLUR: InternedSymbol = InternedSymbol::named("toggle-blur");
const MODES: [&str; 5] = ["translate", "resize", "fade", "transition", "idle"];
const RENDERERS: [RenderBackend; 3] = [
    RenderBackend::Vello,
    RenderBackend::Lyon,
    RenderBackend::VelloHybrid,
];
const CSS: &str = include_str!("frame_probe.css");

struct ProbeState {
    mode: usize,
    renderer: usize,
    blur: bool,
    sample: String,
}

struct ProbeDiagnostics {
    state: Rc<RefCell<ProbeState>>,
    control: AppControl,
}

impl DiagnosticSink for ProbeDiagnostics {
    fn emit(&self, diagnostic: &Diagnostic) {
        match diagnostic.code.as_str() {
            "window.performance" => {
                let mut state = self.state.borrow_mut();
                state.sample.clone_from(&diagnostic.message);
                if MODES[state.mode] != "idle" {
                    self.control.request_frame();
                }
            }
            "window.renderer.changed" => {
                let mut state = self.state.borrow_mut();
                if let Some(index) = RENDERERS.iter().position(|renderer| {
                    diagnostic.message.split_whitespace().next() == Some(renderer.name())
                }) {
                    state.renderer = index;
                }
                state.sample.clear();
                self.control.request_frame();
                eprintln!("[{}] {}", diagnostic.code.as_str(), diagnostic.message);
            }
            _ if diagnostic.severity == obscur_kit::diagnostics::DiagnosticSeverity::Warning => {
                eprintln!("[{}] {}", diagnostic.code.as_str(), diagnostic.message);
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), obscur_kit::AppError> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let has = |flag: &str| args.iter().any(|arg| arg == flag);
    let mode = MODES
        .iter()
        .position(|mode| args.iter().any(|arg| arg.strip_prefix("--") == Some(*mode)))
        .unwrap_or(0);
    let requested_renderer = std::env::var("OBSCUR_RENDERER").ok();
    let renderer = RENDERERS
        .iter()
        .position(|backend| requested_renderer.as_deref() == Some(backend.name()))
        .unwrap_or(0);
    if has("--offscreen") {
        if let Some(name) = requested_renderer.as_deref()
            && name != RENDERERS[renderer].name()
            && name != "classic"
        {
            return Err(obscur_kit::AppError::Gpu(format!(
                "unknown renderer: {name}"
            )));
        }
        return offscreen::run(RENDERERS[renderer], MODES[mode], &args)
            .map_err(obscur_kit::AppError::Gpu);
    }

    let css = probe_stylesheet(MODES[mode], &args);
    let mut app = App::new("Obscur Kit · Frame Probe")
        .app_id("obscur-kit-frame-probe")
        .with_size(720.0, 480.0)
        .with_background_blur(has("--blur"))
        .stylesheet(&css);
    let control = app.control();
    let state = Rc::new(RefCell::new(ProbeState {
        mode,
        renderer,
        blur: has("--blur"),
        sample: String::new(),
    }));
    let stress = has("--stress");
    println!(
        "[probe] mode={} stress={stress}",
        MODES[state.borrow().mode]
    );
    if stress {
        app = app.on_window_waker(|wake| {
            std::thread::spawn(move || {
                while wake() {
                    std::thread::sleep(Duration::from_millis(4));
                }
            });
        });
    }

    let mode_state = state.clone();
    let renderer_state = state.clone();
    let renderer_control = control.clone();
    let blur_state = state.clone();
    app.diagnostics(ProbeDiagnostics {
        state: state.clone(),
        control: control.clone(),
    })
    .on_command(CHANGE_MODE, move || {
        let mut state = mode_state.borrow_mut();
        state.mode = (state.mode + 1) % MODES.len();
        state.sample.clear();
        println!("[probe-mode] {}", MODES[state.mode]);
    })
    .on_command(CHANGE_RENDERER, move || {
        let next = (renderer_state.borrow().renderer + 1) % RENDERERS.len();
        renderer_control.set_renderer(RENDERERS[next]);
    })
    .on_command(TOGGLE_BLUR, move || {
        let mut state = blur_state.borrow_mut();
        state.blur = !state.blur;
        control.set_background_blur(state.blur);
    })
    .view(move || {
        let state = state.borrow();
        let mode = MODES[state.mode];
        let renderer = RENDERERS[state.renderer].name();
        let blur = if state.blur { "on" } else { "off" };
        let controls = Container::new()
            .child(
                Button::new(format!("Mode · {mode}"), CHANGE_MODE)
                    .into_view_node()
                    .class("button"),
            )
            .child(
                Button::new(format!("Renderer · {renderer}"), CHANGE_RENDERER)
                    .into_view_node()
                    .class("button"),
            )
            .child(
                Button::new(format!("Blur · {blur}"), TOGGLE_BLUR)
                    .into_view_node()
                    .class("button"),
            );
        let sample = if state.sample.is_empty() {
            "Wayland presentation · awaiting sample"
        } else {
            state.sample.as_str()
        };
        View::single(
            Container::new()
                .child(controls.into_view_node().class("controls"))
                .child(Text::new(sample).into_view_node())
                .child(tiles(mode).into_view_node().class("scene"))
                .into_view_node()
                .class("root"),
        )
    })
    .run()
}

fn probe_stylesheet(mode: &str, args: &[String]) -> std::borrow::Cow<'static, str> {
    if mode == "transition" && args.iter().any(|arg| arg == "--clip-only") {
        CSS.replace(
            "animation: probe-incoming 250ms ease-in-out infinite alternate;",
            "transform: translateX(400px);",
        )
        .into()
    } else {
        CSS.into()
    }
}

fn tiles(mode: &str) -> Container {
    if mode == "transition" {
        let layer = |class: &str| {
            tiles("idle")
                .into_view_node()
                .class("transition-layer")
                .class(class)
        };
        return Container::new().child(
            Container::new()
                .child(
                    Container::new()
                        .child(layer("transition-underlay"))
                        .into_view_node()
                        .class("transition-clip"),
                )
                .child(layer("transition-incoming"))
                .child(Container::new().into_view_node().class("transition-shadow"))
                .into_view_node()
                .class("transition-stage"),
        );
    }
    let tile_classes = format!("tile {mode}");
    let mut scene = Container::new();
    for _ in 0..6 {
        let mut row = Container::new();
        for _ in 0..8 {
            row = row.child(
                Container::new()
                    .into_view_node()
                    .class(tile_classes.as_str()),
            );
        }
        scene = scene.child(row.into_view_node().class("row"));
    }
    scene
}
