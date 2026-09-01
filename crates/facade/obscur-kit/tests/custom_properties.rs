#![doc = include_str!("custom_properties.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::bootstrap::CssPipeline;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::orchestration::NodeInspection;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::style::computed::{ComputedStyle, ComputedValue};
use obscur_kit::style::property::{Length, PropertyId, PropertyValue, StyleColor, layout_keyword};
use obscur_kit::view::view::{CLASS, STYLE, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("custom_properties.doc/VIEWPORT.md")]
const VIEWPORT: Size = Size {
    width: 400.0,
    height: 300.0,
};

#[doc = include_str!("custom_properties.doc/RED.md")]
const RED: StyleColor = StyleColor::rgba(1.0, 0.0, 0.0, 1.0);
#[doc = include_str!("custom_properties.doc/GREEN.md")]
const GREEN: StyleColor = StyleColor::rgba(0.0, 1.0, 0.0, 1.0);
#[doc = include_str!("custom_properties.doc/BLUE.md")]
const BLUE: StyleColor = StyleColor::rgba(0.0, 0.0, 1.0, 1.0);

#[doc = include_str!("custom_properties.doc/el.md")]
fn el(class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(80))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

#[doc = include_str!("custom_properties.doc/styled.md")]
fn styled(class: &str, style: &str) -> ViewNode {
    el(class, Vec::new()).with_prop(ViewProperty::new(STYLE, ViewValue::Text(style.into())))
}

#[doc = include_str!("custom_properties.doc/screen.md")]
fn screen(root: ViewNode) -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![root]))
}

#[doc = include_str!("custom_properties.doc/settled.md")]
fn settled(sources: &[&str], root: ViewNode) -> (UiRuntime, Vec<NodeInspection>) {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let mut css = CssPipeline::new();
    let sources: Vec<CssSource> = sources
        .iter()
        .map(|text| CssSource::anonymous(*text))
        .collect();
    css.install(&mut runtime, &sources);
    assert!(
        css.errors().is_empty(),
        "the test CSS parses: {:?}",
        css.errors()
    );
    let sheets = css.sheets().to_vec();
    runtime.render_now(&screen(root), &sheets);
    let nodes = runtime.inspect_snapshot().nodes;
    (runtime, nodes)
}

#[doc = include_str!("custom_properties.doc/color_of.md")]
fn color_of(runtime: &UiRuntime, nodes: &[NodeInspection], class: &str) -> StyleColor {
    property_of(runtime, nodes, class, PropertyId::Color)
}

#[doc = include_str!("custom_properties.doc/style_of.md")]
fn style_of<'a>(
    runtime: &'a UiRuntime,
    nodes: &[NodeInspection],
    class: &str,
) -> &'a ComputedStyle {
    let node = nodes
        .iter()
        .find(|node| node.classes.iter().any(|name| name == class))
        .unwrap_or_else(|| panic!("no .{class} node"));
    runtime
        .computed_style(node.node)
        .unwrap_or_else(|| panic!(".{class} has no computed style"))
}

#[doc = include_str!("custom_properties.doc/property_of.md")]
fn property_of(
    runtime: &UiRuntime,
    nodes: &[NodeInspection],
    class: &str,
    property: PropertyId,
) -> StyleColor {
    let node = nodes
        .iter()
        .find(|node| node.classes.iter().any(|name| name == class))
        .unwrap_or_else(|| panic!("no .{class} node"));
    runtime
        .computed_style(node.node)
        .unwrap_or_else(|| panic!(".{class} has no computed style"))
        .get(property)
        .and_then(ComputedValue::as_color)
        .unwrap_or_else(|| panic!(".{class} computed no colour for {property:?}"))
}

#[test]
fn a_var_reference_resolves_to_the_colour_the_root_declared() {
    // The whole chain in one line of CSS: the compiler keeps `--brand`, the
    // pipeline hands it to the runtime, the matcher gives it to the element
    // `:root` selected, and the resolver substitutes it before computing
    // `color`. Break any link and this is black — silently, which is exactly
    // how it went unnoticed.
    let (runtime, nodes) = settled(
        &[":root { --brand: #ff0000 } .a { color: var(--brand) }"],
        el("page", vec![el("a", Vec::new())]),
    );
    assert_eq!(color_of(&runtime, &nodes, "a"), RED);
}

#[test]
fn a_child_inherits_a_custom_property_its_ancestor_declared() {
    // §4.14: custom properties participate in inheritance BEFORE computed-value
    // resolution. `.deep` declares nothing and matches no rule that does; it
    // reads `--brand` purely because `.theme` above it declared one — and it
    // reads `.theme`'s green rather than the root's red, because the nearer
    // declaration is the one it inherits.
    let (runtime, nodes) = settled(
        &[":root { --brand: #ff0000 } \
           .theme { --brand: #00ff00 } \
           .deep { color: var(--brand) } \
           .shallow { color: var(--brand) }"],
        el(
            "page",
            vec![
                el("shallow", Vec::new()),
                el("theme", vec![el("wrap", vec![el("deep", Vec::new())])]),
            ],
        ),
    );
    assert_eq!(
        color_of(&runtime, &nodes, "deep"),
        GREEN,
        "two levels down from the declaration"
    );
    assert_eq!(
        color_of(&runtime, &nodes, "shallow"),
        RED,
        "outside `.theme`, the root's declaration still holds"
    );
}

#[test]
fn a_var_reference_nothing_declared_falls_back_to_the_initial_value() {
    // An unresolvable `var()` makes the declaration invalid at computed-value
    // time, which is `unset`: initial for `background-color` (it does not
    // inherit). The `color` in the same block proves the sheet is wired — the
    // fallback is what the missing name does, not what an unwired pipeline does.
    let (runtime, nodes) = settled(
        &[":root { --brand: #ff0000 } \
           .miss { background-color: var(--nope); color: var(--brand) }"],
        el("page", vec![el("miss", Vec::new())]),
    );
    assert_eq!(
        property_of(&runtime, &nodes, "miss", PropertyId::BackgroundColor),
        StyleColor::TRANSPARENT
    );
    assert_eq!(color_of(&runtime, &nodes, "miss"), RED);
}

#[test]
fn two_stylesheets_each_declaring_a_different_custom_property_both_work() {
    // The interner trap. A compiler is built per source and numbers its symbols
    // from zero, so a custom property named through it would make the FIRST
    // `--name` of every sheet the same property: `--first-brand` and
    // `--second-brand` would be one symbol, and both nodes would come out the
    // same colour — whichever sheet cascaded last.
    let (runtime, nodes) = settled(
        &[
            ":root { --first-brand: #ff0000 } .first { color: var(--first-brand) }",
            ":root { --second-brand: #0000ff } .second { color: var(--second-brand) }",
        ],
        el(
            "page",
            vec![el("first", Vec::new()), el("second", Vec::new())],
        ),
    );
    assert_eq!(color_of(&runtime, &nodes, "first"), RED);
    assert_eq!(color_of(&runtime, &nodes, "second"), BLUE);
}

#[test]
fn a_custom_property_in_the_style_attribute_outranks_the_sheet() {
    // §4.14 origin 5. A custom property is a declaration, so what the author
    // wrote ON the element beats what any selector reached it with — and the
    // `var()` a stylesheet rule wrote reads the element's own answer.
    let (runtime, nodes) = settled(
        &[":root { --brand: #ff0000 } \
           .themed { color: var(--brand) } \
           .plain { color: var(--brand) }"],
        el(
            "page",
            vec![
                styled("themed", "--brand: #0000ff"),
                el("plain", Vec::new()),
            ],
        ),
    );
    assert_eq!(color_of(&runtime, &nodes, "themed"), BLUE);
    assert_eq!(
        color_of(&runtime, &nodes, "plain"),
        RED,
        "the attribute themed its own element, not its siblings"
    );
}

#[test]
fn a_custom_property_declared_inline_is_inherited_by_the_subtree_below_it() {
    // The consequence of doing this in the cascade rather than in the painter:
    // an element themed by its `style` attribute themes everything inside it,
    // which is what makes a per-instance theme worth writing at all.
    let (runtime, nodes) = settled(
        &[":root { --brand: #ff0000 } .child { color: var(--brand) }"],
        el(
            "page",
            vec![
                el("host", vec![el("child", Vec::new())]).with_prop(ViewProperty::new(
                    STYLE,
                    ViewValue::Text("--brand: #0000ff".into()),
                )),
            ],
        ),
    );
    assert_eq!(color_of(&runtime, &nodes, "child"), BLUE);
}

#[test]
fn a_shorthand_whose_value_is_a_var_reference_reaches_the_longhand_it_names() {
    // Consumer audit 2-C. `color: var(--brand)` worked and `background:
    // var(--brand)` did not, because a shorthand's value was split into
    // longhands BEFORE lowering and a `var()` matched no grammar the split
    // knew, so the declaration was discarded whole. An application's every
    // hover and press rule was one such line, and each compiled to an empty
    // block — a design's whole interactive layer, absent.
    let (runtime, nodes) = settled(
        &[":root { --brand: #ff0000; --line: #00ff00 } \
           .a { background: var(--brand); border-bottom: 2px solid var(--line) } \
           .b { border-color: var(--line); border-radius: var(--r) }"],
        el("page", vec![el("a", Vec::new()), el("b", Vec::new())]),
    );
    assert_eq!(
        property_of(&runtime, &nodes, "a", PropertyId::BackgroundColor),
        RED
    );
    assert_eq!(
        property_of(&runtime, &nodes, "a", PropertyId::BorderBottomColor),
        GREEN
    );
    // The width and the style of that same border are still there: the
    // component that could not be resolved yet took only its own longhand.
    assert_eq!(
        runtime
            .computed_style(
                nodes
                    .iter()
                    .find(|node| node.classes.iter().any(|name| name == "a"))
                    .expect("no .a node")
                    .node
            )
            .expect(".a has no computed style")
            .get(PropertyId::BorderBottomWidth)
            .and_then(|value| value.as_length()),
        Some(Length::px(2.0))
    );
    assert_eq!(
        property_of(&runtime, &nodes, "b", PropertyId::BorderLeftColor),
        GREEN
    );
}

#[test]
fn a_substitution_the_property_cannot_carry_computes_to_unset_rather_than_to_absence() {
    // CSS Variables 1 §3: a substitution that is invalid AT COMPUTED-VALUE TIME
    // makes the property compute to `unset` — the inherited value for an
    // inherited property, the initial value for one that is not. Since 2-C the
    // declaration is no longer sunk at compile time, so a `--font-body` holding
    // a length now REACHES `font-family`; kept, it would leave the longhand
    // carrying a value no consumer can read, which is absence — a third
    // behaviour CSS does not offer, and one that paints `.a` in the default
    // face instead of in the face its parent set.
    let (runtime, nodes) = settled(
        &[":root { --font-body: 12px; --loud: solid } \
           .parent { font-family: Georgia; background-color: #ff0000 } \
           .a { font-family: var(--font-body); background-color: var(--loud) }"],
        el("page", vec![el("parent", vec![el("a", Vec::new())])]),
    );
    assert_eq!(
        style_of(&runtime, &nodes, "a")
            .get(PropertyId::FontFamily)
            .map(ComputedValue::value),
        Some(&PropertyValue::font_families(["Georgia"])),
        "`font-family` inherits, so `unset` is the parent's family"
    );
    // The other half of `unset`, on the same node: `background-color` does not
    // inherit, so the parent's red must not reach it either.
    assert_eq!(
        property_of(&runtime, &nodes, "a", PropertyId::BackgroundColor),
        StyleColor::TRANSPARENT,
        "`background-color` does not inherit, so `unset` is its initial"
    );
}

#[test]
fn overflow_as_one_var_reference_reaches_both_axes() {
    // `overflow` is a shorthand over the axis pair, so a value that is one
    // `var()` becomes a reference on `overflow-x` AND on `overflow-y` — nobody
    // had pinned that. What each axis then does is the computed-value-time
    // judgement: a length is no overflow keyword, so both compute to `unset`,
    // and this codebase spells `overflow-*`'s initial `visible` as no stored
    // value at all (its keyword would need an interner).
    let (runtime, nodes) = settled(
        &[":root { --pad: 8px } .a { overflow: var(--pad) }"],
        el("page", vec![el("a", Vec::new())]),
    );
    let style = style_of(&runtime, &nodes, "a");
    assert_eq!(style.get(PropertyId::OverflowX), None);
    assert_eq!(style.get(PropertyId::OverflowY), None);
}

#[test]
fn a_custom_property_keeps_every_component_of_the_value_it_was_written_with() {
    // Audit 2-F. A custom property's value is an arbitrary token sequence (CSS
    // Variables 1 §2): it matches no grammar until it is substituted into a
    // real property. Lowering it as if it were an ordinary value kept only the
    // FIRST component, so `--font-body: Inter, sans-serif` reached
    // `font-family` as a single interned keyword — the fallback family gone,
    // and the symbol one no other sheet can read. `--shadow` lost everything
    // after the `0`.
    let (runtime, nodes) = settled(
        &[
            ":root { --font-body: Inter, sans-serif; --shadow: 0 2px 4px #000000 } \
           .a { font-family: var(--font-body); box-shadow: var(--shadow) } \
           .b { font-family: Inter, sans-serif; box-shadow: 0 2px 4px #000000 }",
        ],
        el("page", vec![el("a", Vec::new()), el("b", Vec::new())]),
    );
    assert_eq!(
        style_of(&runtime, &nodes, "a")
            .get(PropertyId::FontFamily)
            .map(ComputedValue::value),
        Some(&PropertyValue::font_families(["Inter", "sans-serif"])),
        "both families, in the order they were written"
    );
    // `.b` is the same two declarations written literally. Whatever the shadow
    // means, the reference and the literal have to mean it identically — and it
    // has to be a shadow, not two matching absences.
    let literal = style_of(&runtime, &nodes, "b").get(PropertyId::BoxShadow);
    assert!(literal.is_some(), "the literal `box-shadow` compiles");
    assert_eq!(
        style_of(&runtime, &nodes, "a").get(PropertyId::BoxShadow),
        literal,
        "`box-shadow: var(--shadow)` computes what the literal computes"
    );
}

#[doc = include_str!("custom_properties.doc/CLIP_BOXES.md")]
const CLIP_BOXES: &str =
    " .a, .b { width: 50px; height: 20px } .tall { width: 50px; height: 200px }";

#[doc = include_str!("custom_properties.doc/clip_tree.md")]
fn clip_tree() -> ViewNode {
    el(
        "page",
        vec![el("a", vec![el("tall", Vec::new())]), el("b", Vec::new())],
    )
}

#[doc = include_str!("custom_properties.doc/clip_count.md")]
fn clip_count(runtime: &UiRuntime) -> usize {
    runtime
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .filter(|command| matches!(command, PaintCommand::PushClip(_)))
        .count()
}

#[test]
fn an_overflow_keyword_carried_by_a_custom_property_clips_like_the_literal() {
    // `--o: hidden; overflow: var(--o)` was the standing example of a keyword a
    // custom property could not carry: the value was lowered at declaration
    // time by the sheet's own interner, so it matched none of the reserved
    // symbols the layout reads, and the box did not clip. A custom property now
    // keeps its TEXT until the property it lands on lowers it, and `overflow-x`
    // lowers `hidden` through the same reserved-symbol mapper the literal uses.
    let (carried_rt, carried_nodes) = settled(
        &[&format!(
            ":root {{ --o: hidden }} .a {{ overflow: var(--o) }}{CLIP_BOXES}"
        )],
        clip_tree(),
    );
    let (literal_rt, literal_nodes) = settled(
        &[&format!(".a {{ overflow: hidden }}{CLIP_BOXES}")],
        clip_tree(),
    );
    let (visible_rt, _) = settled(&[CLIP_BOXES], clip_tree());

    assert_eq!(
        style_of(&carried_rt, &carried_nodes, "a")
            .get(PropertyId::OverflowX)
            .map(ComputedValue::value),
        Some(&PropertyValue::Keyword(layout_keyword::OVERFLOW_HIDDEN)),
        "the reserved symbol, not one this sheet's interner minted"
    );
    for axis in [PropertyId::OverflowX, PropertyId::OverflowY] {
        assert_eq!(
            style_of(&carried_rt, &carried_nodes, "a").get(axis),
            style_of(&literal_rt, &literal_nodes, "a").get(axis),
            "`overflow: var(--o)` computes what `overflow: hidden` computes"
        );
    }
    // And the consequence the keyword exists for: the box really clips, which
    // is what a symbol no consumer recognises would not have done.
    assert!(
        clip_count(&carried_rt) > clip_count(&visible_rt),
        "the carried `hidden` adds a clip the same tree without it has not"
    );
    assert_eq!(
        clip_count(&carried_rt),
        clip_count(&literal_rt),
        "and exactly the clip the literal `hidden` adds"
    );
}

#[test]
fn a_unitless_zero_carried_by_a_custom_property_is_zero_pixels() {
    // A unitless `0` is a valid length for any length-valued longhand (CSS
    // Values §4.5.3). It used to reach the computed-value step as a NUMBER,
    // which no length property carries, so `width: var(--zero)` computed to
    // `unset` — `auto`, a width the author never asked for. The property the
    // text lands on now lowers `0` to `0px`, exactly as the literal does.
    let (runtime, nodes) = settled(
        &[":root { --zero: 0 } \
           .a { width: var(--zero); margin-left: var(--zero) } \
           .b { width: 0; margin-left: 0 }"],
        el("page", vec![el("a", Vec::new()), el("b", Vec::new())]),
    );
    for property in [PropertyId::Width, PropertyId::MarginLeft] {
        assert_eq!(
            style_of(&runtime, &nodes, "a")
                .get(property)
                .and_then(ComputedValue::as_length),
            Some(Length::px(0.0)),
            "{property:?} carried by a custom property"
        );
        assert_eq!(
            style_of(&runtime, &nodes, "a").get(property),
            style_of(&runtime, &nodes, "b").get(property),
            "{property:?}: the reference computes what the literal computes"
        );
    }
}

#[test]
fn a_word_the_overflow_property_does_not_have_does_not_clip() {
    // The other side of carrying keywords properly. `overflow-x` has five
    // words; `sideways` is none of them, so CSS Variables 1 §3 makes the
    // declaration invalid at computed-value time — `unset`, which is
    // `visible`. It used to be interned instead, and `OverflowAxes` reads every
    // symbol that is not `visible` or `clip` as clipping: a typo in a theme
    // variable clipped the box, which is louder than the value being ignored.
    let (typo_rt, typo_nodes) = settled(
        &[&format!(
            ":root {{ --o: sideways }} .a {{ overflow: var(--o) }}{CLIP_BOXES}"
        )],
        clip_tree(),
    );
    let (visible_rt, _) = settled(&[CLIP_BOXES], clip_tree());

    assert_eq!(
        style_of(&typo_rt, &typo_nodes, "a").get(PropertyId::OverflowX),
        None,
        "no computed value at all, which is how this codebase spells `visible`"
    );
    assert_eq!(
        clip_count(&typo_rt),
        clip_count(&visible_rt),
        "and the frame has the clips it would have had with no `overflow` at all"
    );
}

#[test]
fn the_same_word_written_directly_does_not_clip_either() {
    // The literal spelling of the typo above, which is the commoner way to make
    // it: nobody routes `overflow` through a variable to misspell it. It was
    // left behind when the substitution path learned to reject a word the
    // property does not have, so `overflow-x: sideways` still interned a junk
    // symbol, `OverflowAxes` read anything that is not `visible` or `clip` as
    // clipping, and the box clipped — while `overflow: var(--o)` carrying the
    // very same word did not. One rule, two spellings, two answers.
    let (typo_rt, typo_nodes) = settled(
        &[&format!(".a {{ overflow: sideways }}{CLIP_BOXES}")],
        clip_tree(),
    );
    let (carried_rt, carried_nodes) = settled(
        &[&format!(
            ":root {{ --o: sideways }} .a {{ overflow: var(--o) }}{CLIP_BOXES}"
        )],
        clip_tree(),
    );
    let (visible_rt, _) = settled(&[CLIP_BOXES], clip_tree());

    assert_eq!(
        style_of(&typo_rt, &typo_nodes, "a").get(PropertyId::OverflowX),
        None,
        "a literal word `overflow-x` does not have is no computed value"
    );
    assert_eq!(
        style_of(&typo_rt, &typo_nodes, "a").get(PropertyId::OverflowX),
        style_of(&carried_rt, &carried_nodes, "a").get(PropertyId::OverflowX),
        "and it computes exactly what the same word carried by a var() computes"
    );
    assert_eq!(
        clip_count(&typo_rt),
        clip_count(&visible_rt),
        "the frame has the clips it would have had with no `overflow` at all"
    );
}
