//! End-to-end **headless** capstone for the `obscur-kit` facade.
//!
//! This example drives the whole toolkit through its public umbrella crate only
//! (`obscur_kit::*`) — no individual crate is named — with **no display and no
//! GPU**. It is deterministic and CI-friendly: every stage is exercised through
//! the real pipeline and guarded with `assert!`/`assert_eq!`, so a regression
//! fails the run rather than printing wrong numbers.
//!
//! It walks the full authoring-to-scene path in five acts:
//!
//! 1. **Styling frontend** — author a small CSS stylesheet and compile it via
//!    the CSS frontend (`source -> parse -> compile`), proving the frontend
//!    composes and surfaces unsupported features as diagnostics rather than
//!    crashing.
//! 2. **Components tier** — build a "counter" UI (a container holding a text
//!    label and two buttons) from the primitives + widgets tier.
//! 3. **Pipeline** — boot a deterministic [`HeadlessHost`] via `KitBootstrap`
//!    and render one frame through `reconcile -> style -> layout -> scene`,
//!    asserting the frame reconciled, composed, and is balanced.
//! 4. **Determinism** — render the identical view + inputs again and assert the
//!    display-list commands are byte-identical.
//! 5. **State change** — drive a [`Button`]'s headless state machine (press +
//!    release), rebuild the view, render again, and show the frame id advanced.
//!
//! Run it with:
//! `cargo run -p obscur-kit --example counter_headless`
#![forbid(unsafe_code)]

use obscur_kit::bootstrap::KitBootstrap;
use obscur_kit::css::compiler::{CompiledStylesheet, StylesheetCompiler};
use obscur_kit::css::error::UnsupportedCssFeature;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::identity::InternedSymbol;
use obscur_kit::prelude::*;
use obscur_kit::primitives::{Container, Text};
use obscur_kit::runtime::frame::FrameId;
use obscur_kit::runtime::orchestration::StyleInputs;
use obscur_kit::view::view::{ViewNode, View};
use obscur_kit::widgets::button::Button;

/// A small authored stylesheet that exercises several CSS features at once:
/// a `:root` custom property, a flex container, a `var()` reference, a
/// `:hover`-gated declaration, an `@keyframes` animation, and — on purpose —
/// two things the frontend does not support (`@import` and `float`) so we can
/// show they are reported as diagnostics instead of aborting the compile.
const STYLESHEET: &str = "\
@import \"reset.css\";\n\
:root { --accent: #3366ff; }\n\
.counter { display: flex; opacity: 0.9; }\n\
.counter .value { color: var(--accent); }\n\
button:hover { background-color: #224488; }\n\
button { float: left; }\n\
@keyframes pulse { from { opacity: 0.5; } to { opacity: 1.0; } }\n\
";

/// Commands the two buttons fire when activated (interned symbol handles).
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
    println!("[2] components: built counter view with {} top-level node(s)", view.root().len());
    println!("    label='Count: {count}'  buttons=[\"{}\", \"{}\"]\n", decrement.label(), increment.label());

    // --- Act 3: boot a deterministic host and render one real frame -------
    let mut host = KitBootstrap::new().with_runtime_id(1).headless_host();
    let inputs = StyleInputs::new();

    let first = host.render(&view, &inputs);
    // The first frame must populate the tree, compose a scene, and produce a
    // well-formed (balanced) display list.
    assert!(first.reconciled(), "first frame must reconcile the tree");
    assert!(first.composed(), "a scene is always composed");
    assert!(first.is_balanced(), "the display list must be balanced");
    assert_eq!(first.id(), FrameId::FIRST, "the first frame is FrameId::FIRST");

    let first_commands = first.snapshot().display_list().commands().to_vec();
    print_frame("[3] frame", &first, host.tree().len(), first_commands.len());
    println!();

    // --- Act 4: determinism — same view + inputs => identical display list -
    let second = host.render(&view, &inputs);
    let second_commands = second.snapshot().display_list().commands().to_vec();
    assert_eq!(
        first_commands, second_commands,
        "the same view + inputs must yield a byte-identical display list"
    );
    assert_eq!(second.id(), FrameId::FIRST.next(), "frame ids advance monotonically");
    println!(
        "[4] determinism: re-rendered identical view -> {} commands, byte-identical: {}",
        second_commands.len(),
        first_commands == second_commands
    );
    println!("    frame id advanced {} -> {}\n", first.id().raw(), second.id().raw());

    // --- Act 5: state change — drive the button, rebuild, render again -----
    // A full press/release cycle on an enabled button reports activation.
    increment.press();
    let activated = increment.release();
    assert!(activated, "press + release on an enabled button activates it");
    count += 1;

    let updated_view = build_counter_view(count, &decrement, &increment);
    let third = host.render(&updated_view, &inputs);
    assert!(third.composed(), "the post-change frame still composes a scene");
    assert!(third.is_balanced(), "the post-change display list stays balanced");
    assert!(
        third.id() > second.id(),
        "a new render must advance the frame id"
    );

    let third_commands = third.snapshot().display_list().commands().to_vec();
    println!(
        "[5] state change: '+' activated ({activated}) -> count={count}, view rebuilt"
    );
    print_frame("    frame", &third, host.tree().len(), third_commands.len());
    println!(
        "    frame id advanced {} -> {} (state change drove a new frame)\n",
        second.id().raw(),
        third.id().raw()
    );

    // Keep the compiled stylesheet observably "used" past its summary so the
    // styling frontend is genuinely part of this run, not dead setup.
    assert!(!compiled.declarations.is_empty(), "stylesheet produced declarations");

    println!("== all pipeline invariants held; exiting 0 ==");
}

/// Compiles [`STYLESHEET`] through the CSS frontend (`source -> parse ->
/// compile`) and prints a one-line summary of the resulting native structures.
///
/// This proves the styling frontend composes end-to-end and that recognised-but-
/// unsupported features surface as diagnostics rather than errors or panics.
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
    assert!(compiled.errors.is_empty(), "the authored CSS is syntactically valid");
    assert!(
        !compiled.unsupported.is_empty(),
        "the @import / float features must surface as unsupported diagnostics"
    );
    assert!(!compiled.custom_properties.is_empty(), "--accent is a custom property");
    assert!(!compiled.keyframes.is_empty(), "@keyframes pulse compiles to a track set");

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
    let unsupported: Vec<&str> = compiled.unsupported.iter().map(UnsupportedCssFeature::name).collect();
    println!(
        "    unsupported (reported, not fatal): {} -> {:?}\n",
        compiled.unsupported.len(),
        unsupported
    );

    compiled
}

/// Builds the counter UI: a [`Container`] grouping a text label and the two
/// buttons' element nodes, returned as a single-root [`View`].
///
/// A widget exposes its desired output as a [`View`]; to nest a button inside a
/// container we need its [`ViewNode`]s, which we pull out of the button's root
/// fragment.
fn build_counter_view(count: i64, decrement: &Button, increment: &Button) -> View {
    let label = Text::new(format!("Count: {count}")).into_view_node();
    let container = Container::new()
        .child(label)
        .with_children(button_nodes(decrement))
        .with_children(button_nodes(increment))
        .into_view_node();
    View::single(container)
}

/// Extracts a button's desired [`ViewNode`]s from the [`View`] it renders,
/// so they can be composed as children of another element.
fn button_nodes(button: &Button) -> Vec<ViewNode> {
    button.view().into_fragment().nodes().to_vec()
}

/// Prints a one-line summary of a rendered frame's stats.
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
