#![doc = include_str!("ua_stylesheet_vocabulary.doc/_module.md")]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::default_stylesheet::ua_stylesheet;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{InternedSymbol, NodeId, RuntimeId};
use obscur_kit::okui_compiler::bridge::UiIrViewBridge;
use obscur_kit::okui_compiler::compile::Compiler;
use obscur_kit::okui_syntax::ast::Document;
use obscur_kit::okui_syntax::parser::Parser;
use obscur_kit::primitives::Image;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::style::cascade::CascadeOrigin;
use obscur_kit::style::computed::ComputedValue;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::style::property::{Length, PropertyId};
use obscur_kit::style::selector::{ComplexSelector, CompoundSelector, SimpleSelector};
use obscur_kit::view::view::View;
use obscur_kit::widgets::button::Button;
use obscur_kit::widgets::checkbox::Checkbox;
use obscur_kit::widgets::input::Input;
use obscur_kit::widgets::radio::Radio;
use obscur_kit::widgets::select::Select;
use obscur_kit::widgets::table::{Column, Table};

#[doc = include_str!("ua_stylesheet_vocabulary.doc/CMD.md")]
const CMD: InternedSymbol = InternedSymbol::from_raw(0x4B_00C1);

#[doc = include_str!("ua_stylesheet_vocabulary.doc/FAINT.md")]
const FAINT: (f32, f32, f32) = (
    0x9b as f32 / 255.0,
    0x99 as f32 / 255.0,
    0x8f as f32 / 255.0,
);

#[doc = include_str!("ua_stylesheet_vocabulary.doc/DISABLED_INK.md")]
const DISABLED_INK: (f32, f32, f32) = (
    0xb7 as f32 / 255.0,
    0xb5 as f32 / 255.0,
    0xad as f32 / 255.0,
);

#[doc = include_str!("ua_stylesheet_vocabulary.doc/INK.md")]
const INK: (f32, f32, f32) = (
    0x2b as f32 / 255.0,
    0x2a as f32 / 255.0,
    0x27 as f32 / 255.0,
);

#[doc = include_str!("ua_stylesheet_vocabulary.doc/PAPER.md")]
const PAPER: (f32, f32, f32) = (
    0xfa as f32 / 255.0,
    0xf9 as f32 / 255.0,
    0xf6 as f32 / 255.0,
);

#[doc = include_str!("ua_stylesheet_vocabulary.doc/MARKUP_ONLY_ELEMENTS.md")]
const MARKUP_ONLY_ELEMENTS: &[&str] = &["a", "fieldset"];

#[doc = include_str!("ua_stylesheet_vocabulary.doc/Vocabulary.md")]
#[derive(Debug, Default)]
struct Vocabulary {
    #[doc = include_str!("ua_stylesheet_vocabulary.doc/elements.md")]
    elements: BTreeSet<String>,
    #[doc = include_str!("ua_stylesheet_vocabulary.doc/parts.md")]
    parts: BTreeSet<String>,
    #[doc = include_str!("ua_stylesheet_vocabulary.doc/classes.md")]
    classes: BTreeSet<String>,
}

#[doc = include_str!("ua_stylesheet_vocabulary.doc/rendered.md")]
fn rendered(view: &View) -> UiRuntime {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(600.0, 400.0));
    runtime.set_default_stylesheet(ua_stylesheet());
    runtime.render_now(view, &[]);
    runtime
}

#[doc = include_str!("ua_stylesheet_vocabulary.doc/harvest.md")]
fn harvest(view: &View, into: &mut Vocabulary) {
    for node in rendered(view).inspect_snapshot().nodes {
        if let Some(name) = node.local_name {
            if node.kind == "synthetic" {
                into.parts.insert(name);
            } else {
                into.elements.insert(name);
            }
        }
        into.classes.extend(node.classes);
    }
}

#[doc = include_str!("ua_stylesheet_vocabulary.doc/every_state_the_widgets_can_reach.md")]
fn every_state_the_widgets_can_reach() -> Vec<View> {
    // `select` closes the popup, so the selection is made first and the list
    // opened after it — otherwise the frame carries no option rows at all.
    let mut open_select = Select::new(CMD).with_options(["Alpha", "Beta"]);
    assert!(open_select.select(0), "the first option must be selectable");
    open_select.open();

    // And `open` refuses on a disabled control, so the open-and-disabled state is
    // reached the only way the widget offers: disable a control that is already
    // showing its popup (a form locking while its dropdown is down).
    let mut disabled_open_select = Select::new(CMD).with_options(["Alpha", "Beta"]);
    disabled_open_select.open();
    disabled_open_select.set_disabled(true);

    vec![
        Button::new("Save", CMD).view(),
        Button::new("Save", CMD).disabled(true).view(),
        Input::new(CMD).placeholder("Search").view(),
        Input::new(CMD).placeholder("Search").multiline(true).view(),
        Input::new(CMD).disabled(true).view(),
        Checkbox::new("Enabled", CMD).view(),
        Checkbox::new("Enabled", CMD).checked(true).view(),
        Checkbox::indeterminate("Enabled", CMD).view(),
        Checkbox::new("Enabled", CMD).disabled(true).view(),
        Radio::new("One", CMD).view(),
        Radio::new("One", CMD).selected(true).view(),
        Radio::new("One", CMD).disabled(true).view(),
        Select::new(CMD).with_options(["Alpha"]).view(),
        open_select.view(),
        disabled_open_select.view(),
        Table::new()
            .with_columns([Column::new("Name").sortable(true), Column::new("Note")])
            .row(["Alice", "a"])
            .row(["Bob", "b"])
            .view(),
        Image::new("logo.png").view(),
    ]
}

#[doc = include_str!("ua_stylesheet_vocabulary.doc/toolkit_vocabulary.md")]
fn toolkit_vocabulary() -> Vocabulary {
    let mut vocabulary = Vocabulary::default();
    for view in every_state_the_widgets_can_reach() {
        harvest(&view, &mut vocabulary);
    }
    for tag in MARKUP_ONLY_ELEMENTS {
        harvest(&markup(&format!("<{tag}/>")), &mut vocabulary);
    }
    vocabulary
}

#[doc = include_str!("ua_stylesheet_vocabulary.doc/markup.md")]
fn markup(source: &str) -> View {
    let parsed = Parser::new().parse(source);
    let document = Document::cast(parsed.tree().root().clone()).expect("a document");
    let result = Compiler::new("screen.okui").compile(&document);
    assert!(
        !result.has_errors(),
        "diagnostics: {:?}",
        result.diagnostics()
    );
    UiIrViewBridge::document_to_view(result.document().expect("a lowered document"))
}

#[doc = include_str!("ua_stylesheet_vocabulary.doc/compile.md")]
fn compile(css: &str) -> Stylesheet {
    let source = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&source).parse();
    let compiled = StylesheetCompiler::new()
        .with_origin(CascadeOrigin::SystemDefault)
        .compile(&parsed);
    assert!(compiled.errors.is_empty(), "{:?}", compiled.errors);
    compiled.matchable
}

#[doc = include_str!("ua_stylesheet_vocabulary.doc/compounds.md")]
fn compounds(complex: &ComplexSelector) -> Vec<&CompoundSelector> {
    let mut out = vec![complex.head()];
    out.extend(complex.tail().iter().map(|(_, compound)| compound));
    out
}

#[doc = include_str!("ua_stylesheet_vocabulary.doc/offences.md")]
fn offences(sheet: &Stylesheet, vocabulary: &Vocabulary) -> Vec<String> {
    let mut out = Vec::new();
    for rule in sheet.rules() {
        for complex in rule.selectors().selectors() {
            for compound in compounds(complex) {
                for simple in compound.simples() {
                    match simple {
                        SimpleSelector::Type(name) => {
                            let known = vocabulary.elements.contains(&**name)
                                || MARKUP_ONLY_ELEMENTS.contains(&&**name);
                            if !known {
                                out.push(format!("type `{name}` — nothing builds it"));
                            }
                        }
                        SimpleSelector::Class(name) => {
                            if !vocabulary.classes.contains(&**name) {
                                out.push(format!("class `.{name}` — no widget emits it"));
                            }
                        }
                        SimpleSelector::Part(name) => {
                            if !vocabulary.parts.contains(&**name) {
                                out.push(format!("part `::part({name})` — nothing generates it"));
                            }
                        }
                        SimpleSelector::PseudoClass(pseudo) => {
                            if pseudo.required_state().is_none() {
                                out.push(format!(
                                    "pseudo-class `{pseudo:?}` — the runtime puts no node \
                                     into that state"
                                ));
                            }
                        }
                        SimpleSelector::Id(name) => {
                            out.push(format!("id `#{name}` — a UA sheet must not target one"));
                        }
                        SimpleSelector::Attribute(_) | SimpleSelector::PseudoElement(_) => {
                            out.push(format!("`{simple:?}` — outside the UA sheet's vocabulary"));
                        }
                        SimpleSelector::Universal => {}
                    }
                }
            }
        }
    }
    out
}

#[doc = include_str!("ua_stylesheet_vocabulary.doc/at.md")]
fn at(runtime: &UiRuntime, path: &[usize]) -> NodeId {
    let mut node = runtime.tree().root().expect("a root");
    for &index in path {
        node = runtime.tree().get(node).expect("a live node").child_nodes()[index];
    }
    node
}

#[doc = include_str!("ua_stylesheet_vocabulary.doc/colour.md")]
fn colour(runtime: &UiRuntime, node: NodeId, property: PropertyId) -> Option<(f32, f32, f32)> {
    runtime
        .computed_style(node)
        .and_then(|style| style.get(property))
        .and_then(ComputedValue::as_color)
        .map(|c| (c.r, c.g, c.b))
}

#[doc = include_str!("ua_stylesheet_vocabulary.doc/painted_brush.md")]
fn painted_brush(runtime: &UiRuntime, text: &str) -> Option<(f32, f32, f32)> {
    runtime
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .find_map(|command| match command {
            PaintCommand::DrawText { run, brush, .. } if run.text() == text => brush
                .as_solid()
                .map(|solid| solid.color())
                .map(|c| (c.r, c.g, c.b)),
            _ => None,
        })
}

// --- the guard -------------------------------------------------------------

#[test]
fn every_selector_in_the_ua_sheet_names_something_the_toolkit_builds() {
    // THE guard. A UA sheet styles the *user agent's* own vocabulary; a rule for
    // anything else is invisible work that reads as shipped behaviour. Three rules
    // in this sheet were exactly that before the audit — `table`/`th`/`td`/`tr`
    // against the widget (the parts carried a private `role` property), `img`
    // against the primitive (it carried no name at all), and `.placeholder`
    // against a class nothing emits, which painted the hint in full ink.
    //
    // The vocabulary is HARVESTED, not declared: every widget and primitive is
    // built in every state that emits a distinct name or class, rendered through a
    // real runtime, and read back off the retained tree. The one declared part is
    // `MARKUP_ONLY_ELEMENTS`, and the test below proves each entry reaches the
    // tree.
    let vocabulary = toolkit_vocabulary();
    let found = offences(&ua_stylesheet(), &vocabulary);
    assert!(
        found.is_empty(),
        "the UA sheet styles things nothing builds:\n  {}\nharvested elements: {:?}\n\
         harvested classes: {:?}\nharvested parts: {:?}",
        found.join("\n  "),
        vocabulary.elements,
        vocabulary.classes,
        vocabulary.parts,
    );
}

#[test]
fn the_guard_rejects_a_rule_for_a_name_nothing_builds() {
    // The guard's teeth, kept rather than demonstrated once and thrown away: a
    // test that only ever passes proves nothing about what it would catch. Each
    // line below is a way the sheet could go stale, and each is rejected.
    let vocabulary = toolkit_vocabulary();
    let found = offences(
        &compile(
            "bogus { color: #000000; }\
             .no-such-part { color: #000000; }\
             input::part(nonesuch) { color: #000000; }\
             #the-one-input { color: #000000; }\
             button:first-child { color: #000000; }",
        ),
        &vocabulary,
    );
    let report = found.join("\n  ");
    for expected in [
        "type `bogus`",
        "class `.no-such-part`",
        "part `::part(nonesuch)`",
        "id `#the-one-input`",
        "pseudo-class",
    ] {
        assert!(
            report.contains(expected),
            "the guard must reject {expected}: {report}"
        );
    }
}

#[test]
fn the_markup_only_vocabulary_really_reaches_the_tree() {
    // The declared half of the vocabulary, and the reason it is small. `a` and
    // `fieldset` are HTML element names no widget and no primitive produces — the
    // sheet styles them for hand-written markup, where every tag becomes its own
    // local name. That is a real route, so the rules are alive; it is also the
    // only escape hatch the guard has, so each entry is proved rather than
    // asserted, and adding one is a deliberate act.
    for tag in MARKUP_ONLY_ELEMENTS {
        let runtime = rendered(&markup(&format!("<{tag}/>")));
        let names: Vec<Option<String>> = runtime
            .inspect_snapshot()
            .nodes
            .into_iter()
            .map(|node| node.local_name)
            .collect();
        assert!(
            names.iter().any(|name| name.as_deref() == Some(*tag)),
            "<{tag}/> must reach the tree named `{tag}`: {names:?}"
        );
    }
}

// --- the three rules the audit found, each pinned where it shows -----------

#[test]
fn the_placeholder_hint_is_painted_in_the_faint_ink() {
    // The one dead rule whose deadness was VISIBLE. §4.16 makes the placeholder a
    // synthetic, paint-only part generated from the field's reserved attribute —
    // `Input::view` says so outright, "the placeholder is NOT a child" — and its
    // colour is resolved through `part_specificity_for`, which reads a `::part()`
    // subject and nothing else. The sheet spelled the rule `.placeholder`, a
    // CLASS, and nothing the toolkit builds carries that class: the hint was
    // painted in the full `#2b2a27` ink, indistinguishable from a typed value,
    // which is the exact thing a placeholder exists not to be.
    for multiline in [false, true] {
        let field = Input::new(CMD).placeholder("Search").multiline(multiline);
        let runtime = rendered(&field.view());
        assert_eq!(
            painted_brush(&runtime, "Search"),
            Some(FAINT),
            "the {} field's hint must paint in the faint ink #9b998f",
            if multiline { "textarea" } else { "input" },
        );
    }
}

#[test]
fn the_image_primitive_answers_to_the_ua_img_rule() {
    // `<img>` in markup and `Image::new(…)` in Rust are one built-in —
    // `builtin_component_id("img")` is `Image::COMPONENT_ID` — but the tag
    // contributed a local name and the primitive contributed none, so the sheet's
    // `img { … }` reached one of the two. Same id, same name, one rule now.
    let widget = rendered(&Image::new("logo.png").view());
    let authored = rendered(&markup("<img src=\"logo.png\"/>"));
    for (label, runtime) in [("Image::new", &widget), ("<img>", &authored)] {
        assert_eq!(
            runtime
                .tree()
                .get(at(runtime, &[0]))
                .and_then(|record| record.attributes().local_name()),
            Some("img"),
            "{label} must reach the tree named `img`"
        );
    }
}

#[test]
fn an_authored_type_attribute_does_not_cost_the_element_its_ua_styling() {
    // The vocabulary work's *first* fault, measured where the audit stated it —
    // against the real UA sheet rather than a one-property probe. `TYPE` used to
    // carry both the element's name and the authored `type="…"`, so
    // `<button type="submit">` was an element whose CSS type was `submit`: it
    // matched no `button { … }` rule and lost the WHOLE of the sheet's button look
    // — background, ink, border colour, font size — with no diagnostic. Every one
    // of those four is asserted, because "all its UA styling" is the claim.
    let runtime = rendered(&markup(
        "<button type=\"submit\" class=\"b\">\"OK\"</button>",
    ));
    let button = runtime
        .inspect_snapshot()
        .nodes
        .into_iter()
        .find(|node| node.classes.iter().any(|name| name == "b"))
        .expect("the authored button");
    assert_eq!(button.local_name.as_deref(), Some("button"));
    assert_eq!(
        colour(&runtime, button.node, PropertyId::BackgroundColor),
        Some(PAPER),
        "`button {{ background: #faf9f6 }}` must reach a <button type=submit>"
    );
    assert_eq!(
        colour(&runtime, button.node, PropertyId::Color),
        Some(INK),
        "`button {{ color: #2b2a27 }}` must reach it too"
    );
    assert_eq!(
        colour(&runtime, button.node, PropertyId::BorderTopColor),
        Some(INK),
        "and the border colour the `border` shorthand sets"
    );
    assert_eq!(
        runtime
            .computed_style(button.node)
            .and_then(|style| style.get(PropertyId::FontSize))
            .and_then(ComputedValue::as_length),
        Some(Length::px(14.0)),
        "and the font size"
    );
}

#[test]
fn a_disabled_selects_options_read_muted() {
    // The third rule, and the one that was aspirational rather than merely
    // misspelled: it read `.select-option.disabled`, a compound with a `disabled`
    // CLASS, and `Select`'s options are plain labels — there is no per-option
    // disabled state for anything to emit. The state the widget *does* have is a
    // disabled control, spelled the way the sheet's `.select-arrow` rule beside it
    // already spells it, so the rule now says what it always meant.
    // `Select::open` refuses on a disabled control, so the one reachable
    // open-and-disabled state is a control disabled while its popup is already
    // down — a form locking mid-interaction.
    let mut select = Select::new(CMD).with_options(["Alpha", "Beta"]);
    select.open();
    select.set_disabled(true);
    let runtime = rendered(&select.view());
    // child 0 is the `.select-header`, child 1 the `.select-list`, whose children
    // are the option rows.
    let option = at(&runtime, &[0, 0, 1, 0]);
    assert_eq!(
        colour(&runtime, option, PropertyId::Color),
        Some(DISABLED_INK),
        "an option of a disabled select must read in the disabled ink"
    );

    // And only while disabled: an enabled select's options keep the ordinary ink,
    // so this is the control's state and not a rule that reaches every option.
    let mut enabled = Select::new(CMD).with_options(["Alpha", "Beta"]);
    enabled.open();
    let runtime = rendered(&enabled.view());
    assert_ne!(
        colour(&runtime, at(&runtime, &[0, 0, 1, 0]), PropertyId::Color),
        Some(DISABLED_INK)
    );
}
