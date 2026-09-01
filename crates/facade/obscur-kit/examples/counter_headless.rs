#![doc = include_str!("counter_headless.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::bootstrap::KitBootstrap;
use obscur_kit::css::compiler::{CompiledStylesheet, StylesheetCompiler, lower_custom_value};
use obscur_kit::css::error::UnsupportedCssFeature;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::identity::InternedSymbol;
use obscur_kit::prelude::*;
use obscur_kit::primitives::{Container, Text};
use obscur_kit::runtime::frame::FrameId;
use obscur_kit::view::view::{CLASS, View, ViewNode, ViewProperty, ViewValue};
use obscur_kit::widgets::button::Button;

#[doc = include_str!("counter_headless.doc/STYLESHEET.md")]
const STYLESHEET: &str = "\
@import \"reset.css\";\n\
:root { --accent: #3366ff; }\n\
.counter   { display: flex; opacity: 0.9; background-color: #1e2636; width: 100%; height: 100%; }\n\
.bar       { width: 100%; }\n\
.label     { background-color: #3a4560; height: 50%; }\n\
.decrement { background-color: #d9534f; height: 25%; }\n\
.increment { background-color: #5cb85c; height: 25%; }\n\
.counter .value { color: var(--accent); }\n\
button:hover { background-color: #224488; }\n\
button { float: left; }\n\
@keyframes pulse { from { opacity: 0.5; } to { opacity: 1.0; } }\n\
";

#[doc = include_str!("counter_headless.doc/CMD_DECREMENT.md")]
const CMD_DECREMENT: InternedSymbol = InternedSymbol::from_raw(0x1001);
const CMD_INCREMENT: InternedSymbol = InternedSymbol::from_raw(0x1002);

fn main() {
    println!("== obscur-kit headless counter ==");
    println!("(full pipeline via the public facade — no window, no GPU)\n");

    // --- Act 1: the styling frontend composes -----------------------------
    let compiled = compile_stylesheet();

    // --- Act 2: build the counter UI from the components tier --------------
    // The two buttons own their own headless press/release state.
    let decrement = Button::new("-", CMD_DECREMENT);
    let mut increment = Button::new("+", CMD_INCREMENT);
    let mut count: i64 = 0;

    let view = build_counter_view(count, &decrement, &increment);
    println!(
        "[2] components: built counter view with {} top-level node(s)",
        view.root().len()
    );
    println!(
        "    label='Count: {count}'  buttons=[\"{}\", \"{}\"]\n",
        decrement.label(),
        increment.label()
    );

    // --- Act 3: boot a deterministic host and render one real frame -------
    // The compiled stylesheet's `matchable` form is the list of (selector ->
    // declarations) rules the runtime matches against the reconciled tree;
    // `render_styled` runs reconcile -> selector match -> style -> layout ->
    // scene, so the `.counter` rule paints the tagged container.
    let mut host = KitBootstrap::new().with_runtime_id(1).headless_host();
    // `--accent` reaches the runtime as the TEXT it was declared with, because a
    // custom property has no grammar until a `var()` puts it on a property. This
    // host wires its own sheets rather than going through `CssPipeline`, so it
    // hands the runtime the grammar itself — without it `color: var(--accent)`
    // is invalid at computed-value time and the value paints in the inherited
    // colour.
    host.runtime_mut()
        .set_custom_value_parser(std::rc::Rc::new(lower_custom_value));
    let sheets = std::slice::from_ref(&compiled.matchable);

    let first = host.render_styled(&view, sheets);
    // The first frame must populate the tree, compose a scene, and produce a
    // well-formed (balanced) display list.
    assert!(first.reconciled(), "first frame must reconcile the tree");
    assert!(first.composed(), "a scene is always composed");
    assert!(first.is_balanced(), "the display list must be balanced");
    assert_eq!(
        first.id(),
        FrameId::FIRST,
        "the first frame is FrameId::FIRST"
    );

    let first_commands = first.snapshot().display_list().commands().to_vec();
    // The `.counter { background-color; width; height }` rule matched the tagged
    // container and produced a real fill — the display list is non-empty.
    assert!(
        !first_commands.is_empty(),
        "the matched .counter background must emit at least one paint command"
    );
    print_frame("[3] frame", &first, host.tree().len(), first_commands.len());
    println!();

    // --- Act 4: determinism — same view + sheets => identical display list -
    let second = host.render_styled(&view, sheets);
    let second_commands = second.snapshot().display_list().commands().to_vec();
    assert_eq!(
        first_commands, second_commands,
        "the same view + sheets must yield a byte-identical display list"
    );
    assert_eq!(
        second.id(),
        FrameId::FIRST.next(),
        "frame ids advance monotonically"
    );
    println!(
        "[4] determinism: re-rendered identical view -> {} commands, byte-identical: {}",
        second_commands.len(),
        first_commands == second_commands
    );
    println!(
        "    frame id advanced {} -> {}\n",
        first.id().raw(),
        second.id().raw()
    );

    // --- Act 5: state change — drive the button, rebuild, render again -----
    // A full press/release cycle on an enabled button reports activation.
    increment.press();
    let activated = increment.release();
    assert!(
        activated,
        "press + release on an enabled button activates it"
    );
    count += 1;

    let updated_view = build_counter_view(count, &decrement, &increment);
    let third = host.render_styled(&updated_view, sheets);
    assert!(
        third.composed(),
        "the post-change frame still composes a scene"
    );
    assert!(
        third.is_balanced(),
        "the post-change display list stays balanced"
    );
    assert!(
        third.id() > second.id(),
        "a new render must advance the frame id"
    );

    let third_commands = third.snapshot().display_list().commands().to_vec();
    println!("[5] state change: '+' activated ({activated}) -> count={count}, view rebuilt");
    print_frame("    frame", &third, host.tree().len(), third_commands.len());
    println!(
        "    frame id advanced {} -> {} (state change drove a new frame)\n",
        second.id().raw(),
        third.id().raw()
    );

    // Keep the compiled stylesheet observably "used" past its summary so the
    // styling frontend is genuinely part of this run, not dead setup.
    assert!(
        !compiled.declarations.is_empty(),
        "stylesheet produced declarations"
    );

    println!("== all pipeline invariants held; exiting 0 ==");
}

#[doc = include_str!("counter_headless.doc/compile_stylesheet.md")]
fn compile_stylesheet() -> CompiledStylesheet {
    // 1. Wrap the authored text in a source (carries id + name for diagnostics).
    let source = CssSource::anonymous(STYLESHEET);
    // 2. Parse into a resilient AST (malformed input recovers; it never panics).
    let parsed = StylesheetParser::new(&source).parse();
    // 3. Lower the AST into obscur-style / obscur-motion native structures. The
    //    compiler consumes its own interner into the result, so `compile` takes
    //    `self` by value.
    let compiled = StylesheetCompiler::new().compile(&parsed);

    // The stylesheet is syntactically clean, so parsing yields no errors, yet it
    // deliberately contains features the frontend does not support.
    assert!(
        compiled.errors.is_empty(),
        "the authored CSS is syntactically valid"
    );
    assert!(
        !compiled.unsupported.is_empty(),
        "the @import / float features must surface as unsupported diagnostics"
    );
    assert!(
        !compiled.custom_properties.is_empty(),
        "--accent is a custom property"
    );
    assert!(
        !compiled.keyframes.is_empty(),
        "@keyframes pulse compiles to a track set"
    );

    println!(
        "[1] css frontend: {} rule declaration(s), {} custom prop(s), {} keyframe rule(s), \
{} transition(s), {} conditional group(s)",
        compiled.declarations.len(),
        compiled.custom_properties.len(),
        compiled.keyframes.len(),
        compiled.transitions.len(),
        compiled.conditional_groups.len(),
    );
    // List the features that were recognised but not supported, by name.
    let unsupported: Vec<&str> = compiled
        .unsupported
        .iter()
        .map(UnsupportedCssFeature::name)
        .collect();
    println!(
        "    unsupported (reported, not fatal): {} -> {:?}\n",
        compiled.unsupported.len(),
        unsupported
    );

    compiled
}

#[doc = include_str!("counter_headless.doc/build_counter_view.md")]
fn build_counter_view(count: i64, decrement: &Button, increment: &Button) -> View {
    // Each child is wrapped in a container carrying a class the stylesheet
    // paints; the `.counter` panel stacks them (flow layout, top-to-bottom).
    let label = tagged(
        "bar label",
        vec![Text::new(format!("Count: {count}")).into_view_node()],
    );
    let minus = tagged("bar decrement", button_nodes(decrement));
    let plus = tagged("bar increment", button_nodes(increment));
    let container = Container::new()
        .child(label)
        .child(minus)
        .child(plus)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("counter".into())));
    View::single(container)
}

#[doc = include_str!("counter_headless.doc/tagged.md")]
fn tagged(class: &str, children: Vec<ViewNode>) -> ViewNode {
    Container::new()
        .with_children(children)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

#[doc = include_str!("counter_headless.doc/button_nodes.md")]
fn button_nodes(button: &Button) -> Vec<ViewNode> {
    button.view().into_fragment().nodes().to_vec()
}

#[doc = include_str!("counter_headless.doc/print_frame.md")]
fn print_frame(tag: &str, frame: &FrameResult, tree_nodes: usize, commands: usize) {
    println!(
        "{tag}: id={} reconciled={} composed={} balanced={} | tree nodes={tree_nodes} \
display-list commands={commands}",
        frame.id().raw(),
        frame.reconciled(),
        frame.composed(),
        frame.is_balanced(),
    );
}
