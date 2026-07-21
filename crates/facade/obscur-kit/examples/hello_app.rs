//! The **declarative** windowed counter — the `windowed_counter.rs` capstone
//! rewritten on the one-call [`obscur_kit::App`] entry point (spec §1.4).
//!
//! `windowed_counter.rs` spells out ~890 lines to open one window: a winit
//! `ApplicationHandler`, the wgpu `Instance → Surface → Adapter → Device → Queue`
//! chain, the multi-font fallback stack + renderer font hand-off, the CSS compile,
//! swapchain resize/lost recovery, the present quad, and the pointer/key/IME arms.
//! [`App`] composes all of that from the kit's own facilities, so the app code
//! below is just the counter's state, view builder, CSS, and command handlers —
//! the *same behaviour*, an order of magnitude less boilerplate.
//!
//! Run it on a real machine (a display + a GPU adapter are required):
//! `cargo run -p obscur-kit --example hello_app --features full`
//!
//! Clicking the green bar increments the on-screen count and the red bar
//! decrements it; hovering a bar lights its `:hover` style. It cannot run in the
//! headless dev/CI environment (no display, no GPU), so there it is only compiled.
#![forbid(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;

use obscur_kit::App;
use obscur_kit::identity::InternedSymbol;
use obscur_kit::primitives::{Container, Text};
use obscur_kit::view::view::{CLASS, View, ViewNode, ViewProperty, ViewValue};
use obscur_kit::widgets::button::Button;

/// The two button commands (interned symbol handles), identical to the original.
const CMD_DECREMENT: InternedSymbol = InternedSymbol::from_raw(0x1001);
const CMD_INCREMENT: InternedSymbol = InternedSymbol::from_raw(0x1002);

/// The authored stylesheet — byte-identical to `windowed_counter.rs`, so the look
/// and the `:hover` behaviour are the same.
const COUNTER_CSS: &str = "\
.counter         { background-color: #1e2636; width: 100%; height: 100%; }\n\
.bar             { width: 100%; }\n\
.label           { background-color: #2b3348; height: 50%; }\n\
.decrement       { background-color: #d9534f; height: 25%; }\n\
.increment       { background-color: #5cb85c; height: 25%; }\n\
.decrement:hover { background-color: #ef7b77; }\n\
.increment:hover { background-color: #7fd39b; }\n\
";

fn main() -> Result<(), obscur_kit::AppError> {
    // The shared counter, mutated by the click handlers and read by the reactive
    // view builder — the canonical `Rc<Cell<i32>>` bridge, exactly as before.
    let count = Rc::new(Cell::new(0));

    let view_count = count.clone();
    let inc = count.clone();
    let dec = count.clone();

    App::new("obscur-kit — hello_app (declarative counter)")
        // A distinct Wayland `app_id` / X11 `WM_CLASS`, so a compositor rule can
        // target this example alone (the title is human-readable and may change;
        // the app_id is the stable identity rules match on).
        .app_id("obscur-kit-hello-app")
        .with_size(480.0, 240.0)
        .stylesheet(COUNTER_CSS)
        .view(move || build_counter_view(view_count.get()))
        .on_command(CMD_INCREMENT, move || inc.set(inc.get() + 1))
        .on_command(CMD_DECREMENT, move || dec.set(dec.get() - 1))
        .run()
}

/// Builds the counter UI from `count` (unchanged from `windowed_counter.rs`): a
/// dark `.counter` panel stacking a label bar and the two real [`Button`] bars.
fn build_counter_view(count: i32) -> View {
    let label = tagged(
        "bar label",
        vec![Text::new(format!("Count 数字: {count}")).into_view_node()],
    );
    let minus = button_bar("bar decrement", "-", CMD_DECREMENT);
    let plus = button_bar("bar increment", "+", CMD_INCREMENT);
    let container = Container::new()
        .child(label)
        .child(minus)
        .child(plus)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("counter".into())));
    View::single(container)
}

/// Wraps `children` in a container tagged with the space-separated `class`.
fn tagged(class: &str, children: Vec<ViewNode>) -> ViewNode {
    Container::new()
        .with_children(children)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

/// A real [`Button`] bound to `command`, rendered as a sizing bar (the sizing
/// class is on the button's own node so the click resolves to it — see the
/// original example's note on ancestor-chain resolution).
fn button_bar(class: &str, label: &str, command: InternedSymbol) -> ViewNode {
    Button::new(label, command)
        .view()
        .into_fragment()
        .nodes()
        .to_vec()
        .remove(0)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}
