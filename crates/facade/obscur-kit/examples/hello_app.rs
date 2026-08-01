#![doc = include_str!("hello_app.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;

use obscur_kit::App;
use obscur_kit::identity::InternedSymbol;
use obscur_kit::primitives::{Container, Text};
use obscur_kit::view::view::{View, ViewNode};
use obscur_kit::widgets::button::Button;

#[doc = include_str!("hello_app.doc/CMD_DECREMENT.md")]
const CMD_DECREMENT: InternedSymbol = InternedSymbol::named("decrement");
const CMD_INCREMENT: InternedSymbol = InternedSymbol::named("increment");

#[doc = include_str!("hello_app.doc/COUNTER_CSS.md")]
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

#[doc = include_str!("hello_app.doc/build_counter_view.md")]
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
        .class("counter");
    View::single(container)
}

#[doc = include_str!("hello_app.doc/tagged.md")]
fn tagged(class: &str, children: Vec<ViewNode>) -> ViewNode {
    Container::new()
        .with_children(children)
        .into_view_node()
        .class(class)
}

#[doc = include_str!("hello_app.doc/button_bar.md")]
fn button_bar(class: &str, label: &str, command: InternedSymbol) -> ViewNode {
    Button::new(label, command).into_view_node().class(class)
}
