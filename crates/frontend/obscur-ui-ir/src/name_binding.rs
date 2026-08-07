#![doc = include_str!("name_binding.doc/_module.md")]

use obscur_identity::{ComponentId, InternedSymbol};

use crate::document::{UiIrExpression, UiIrValue};

// --- Reserved attribute symbols (mirror `obscur_view::view::style_attribute`). ---

#[doc = include_str!("name_binding.doc/CLASS.md")]
pub const CLASS: InternedSymbol = InternedSymbol::from_raw(0x0000_0001);
#[doc = include_str!("name_binding.doc/ID.md")]
pub const ID: InternedSymbol = InternedSymbol::from_raw(0x0000_0002);
#[doc = include_str!("name_binding.doc/TYPE.md")]
pub const TYPE: InternedSymbol = InternedSymbol::from_raw(0x0000_0003);
#[doc = include_str!("name_binding.doc/SRC.md")]
pub const SRC: InternedSymbol = InternedSymbol::from_raw(0x0000_0004);
#[doc = include_str!("name_binding.doc/DISABLED.md")]
pub const DISABLED: InternedSymbol = InternedSymbol::from_raw(0x0000_0005);
#[doc = include_str!("name_binding.doc/CHECKED.md")]
pub const CHECKED: InternedSymbol = InternedSymbol::from_raw(0x0000_0006);
#[doc = include_str!("name_binding.doc/PLACEHOLDER.md")]
pub const PLACEHOLDER: InternedSymbol = InternedSymbol::from_raw(0x0000_0009);
#[doc = include_str!("name_binding.doc/LOCAL_NAME.md")]
pub const LOCAL_NAME: InternedSymbol = InternedSymbol::from_raw(0x0000_000A);
#[doc = include_str!("name_binding.doc/STYLE.md")]
pub const STYLE: InternedSymbol = InternedSymbol::from_raw(0x0000_000B);

#[doc = include_str!("name_binding.doc/TEST_ID.md")]
pub const TEST_ID: InternedSymbol = InternedSymbol::from_raw(0x0000_0201);

#[doc = include_str!("name_binding.doc/AUTOMATION_ID.md")]
pub const AUTOMATION_ID: InternedSymbol = InternedSymbol::from_raw(0x0000_0202);

#[doc = include_str!("name_binding.doc/SLOT.md")]
pub const SLOT: InternedSymbol = InternedSymbol::from_raw(0x0000_0301);

// --- Reserved control symbols (mirror `obscur_view::view::slider` and
//     `obscur_widgets::progress`). ---

#[doc = include_str!("name_binding.doc/SLIDER_MIN.md")]
pub const SLIDER_MIN: InternedSymbol = InternedSymbol::from_raw(0x0000_0501);
#[doc = include_str!("name_binding.doc/SLIDER_MAX.md")]
pub const SLIDER_MAX: InternedSymbol = InternedSymbol::from_raw(0x0000_0502);
#[doc = include_str!("name_binding.doc/SLIDER_STEP.md")]
pub const SLIDER_STEP: InternedSymbol = InternedSymbol::from_raw(0x0000_0503);
#[doc = include_str!("name_binding.doc/SLIDER_LOW.md")]
pub const SLIDER_LOW: InternedSymbol = InternedSymbol::from_raw(0x0000_0504);

#[doc = include_str!("name_binding.doc/PROGRESS_VALUE.md")]
pub const PROGRESS_VALUE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0702);

#[doc = include_str!("name_binding.doc/SLOT_OUTLET.md")]
pub const SLOT_OUTLET: ComponentId = ComponentId::from_raw(0x00B0_0001);

#[doc = include_str!("name_binding.doc/OUTLET_TAG.md")]
pub const OUTLET_TAG: &str = "slot";

// --- Reserved event symbols (mirror `obscur_view::view::event_name`). ---

#[doc = include_str!("name_binding.doc/EVENT_CLICK.md")]
pub const EVENT_CLICK: InternedSymbol = InternedSymbol::from_raw(0x0000_0101);
#[doc = include_str!("name_binding.doc/EVENT_INPUT.md")]
pub const EVENT_INPUT: InternedSymbol = InternedSymbol::from_raw(0x0000_0102);
#[doc = include_str!("name_binding.doc/EVENT_CHANGE.md")]
pub const EVENT_CHANGE: InternedSymbol = InternedSymbol::from_raw(0x0000_0103);
#[doc = include_str!("name_binding.doc/EVENT_LONG_PRESS.md")]
pub const EVENT_LONG_PRESS: InternedSymbol = InternedSymbol::from_raw(0x0000_0104);
#[doc = include_str!("name_binding.doc/EVENT_KEY_DOWN.md")]
pub const EVENT_KEY_DOWN: InternedSymbol = InternedSymbol::from_raw(0x0000_0105);
#[doc = include_str!("name_binding.doc/EVENT_KEY_UP.md")]
pub const EVENT_KEY_UP: InternedSymbol = InternedSymbol::from_raw(0x0000_0106);

#[doc = include_str!("name_binding.doc/EVENT_POINTER_DOWN.md")]
pub const EVENT_POINTER_DOWN: InternedSymbol = InternedSymbol::from_raw(0x0000_0107);
#[doc = include_str!("name_binding.doc/EVENT_POINTER_UP.md")]
pub const EVENT_POINTER_UP: InternedSymbol = InternedSymbol::from_raw(0x0000_0108);
#[doc = include_str!("name_binding.doc/EVENT_POINTER_MOVE.md")]
pub const EVENT_POINTER_MOVE: InternedSymbol = InternedSymbol::from_raw(0x0000_0109);
#[doc = include_str!("name_binding.doc/EVENT_POINTER_CANCEL.md")]
pub const EVENT_POINTER_CANCEL: InternedSymbol = InternedSymbol::from_raw(0x0000_010A);
#[doc = include_str!("name_binding.doc/EVENT_POINTER_ENTER.md")]
pub const EVENT_POINTER_ENTER: InternedSymbol = InternedSymbol::from_raw(0x0000_010B);
#[doc = include_str!("name_binding.doc/EVENT_POINTER_LEAVE.md")]
pub const EVENT_POINTER_LEAVE: InternedSymbol = InternedSymbol::from_raw(0x0000_010C);

#[doc = include_str!("name_binding.doc/EVENT_WHEEL.md")]
pub const EVENT_WHEEL: InternedSymbol = InternedSymbol::from_raw(0x0000_010D);

#[doc = include_str!("name_binding.doc/EVENT_FOCUS.md")]
pub const EVENT_FOCUS: InternedSymbol = InternedSymbol::from_raw(0x0000_010E);
#[doc = include_str!("name_binding.doc/EVENT_BLUR.md")]
pub const EVENT_BLUR: InternedSymbol = InternedSymbol::from_raw(0x0000_010F);

#[doc = include_str!("name_binding.doc/EVENT_CONTEXT_MENU.md")]
pub const EVENT_CONTEXT_MENU: InternedSymbol = InternedSymbol::from_raw(0x0000_0110);

#[doc = include_str!("name_binding.doc/RESERVED_EVENTS.md")]
pub const RESERVED_EVENTS: &[InternedSymbol] = &[
    EVENT_CLICK,
    EVENT_INPUT,
    EVENT_CHANGE,
    EVENT_LONG_PRESS,
    EVENT_KEY_DOWN,
    EVENT_KEY_UP,
    EVENT_POINTER_DOWN,
    EVENT_POINTER_UP,
    EVENT_POINTER_MOVE,
    EVENT_POINTER_CANCEL,
    EVENT_POINTER_ENTER,
    EVENT_POINTER_LEAVE,
    EVENT_WHEEL,
    EVENT_FOCUS,
    EVENT_BLUR,
    EVENT_CONTEXT_MENU,
];

// --- Primitive component ids (mirror `obscur_primitives::*::COMPONENT_ID`). ---

#[doc = include_str!("name_binding.doc/CONTAINER.md")]
pub const CONTAINER: ComponentId = ComponentId::from_raw(0x00C0_0001);
#[doc = include_str!("name_binding.doc/IMAGE.md")]
pub const IMAGE: ComponentId = ComponentId::from_raw(0x00C0_0003);
#[doc = include_str!("name_binding.doc/SCROLL.md")]
pub const SCROLL: ComponentId = ComponentId::from_raw(0x00C0_0004);
#[doc = include_str!("name_binding.doc/CANVAS.md")]
pub const CANVAS: ComponentId = ComponentId::from_raw(0x00C0_0005);
#[doc = include_str!("name_binding.doc/PORTAL.md")]
pub const PORTAL: ComponentId = ComponentId::from_raw(0x00C0_0006);
#[doc = include_str!("name_binding.doc/LAYER.md")]
pub const LAYER: ComponentId = ComponentId::from_raw(0x00C0_0007);
#[doc = include_str!("name_binding.doc/FOREIGN_SURFACE.md")]
pub const FOREIGN_SURFACE: ComponentId = ComponentId::from_raw(0x00C0_0008);

// --- Widget component ids (mirror `obscur_widgets::*::COMPONENT_ID`). ---

#[doc = include_str!("name_binding.doc/BUTTON.md")]
pub const BUTTON: ComponentId = ComponentId::from_raw(0x00D0_0001);
#[doc = include_str!("name_binding.doc/INPUT.md")]
pub const INPUT: ComponentId = ComponentId::from_raw(0x00D0_0002);
#[doc = include_str!("name_binding.doc/CHECKBOX.md")]
pub const CHECKBOX: ComponentId = ComponentId::from_raw(0x00D0_0003);
#[doc = include_str!("name_binding.doc/SELECT.md")]
pub const SELECT: ComponentId = ComponentId::from_raw(0x00D0_0004);
#[doc = include_str!("name_binding.doc/MENU.md")]
pub const MENU: ComponentId = ComponentId::from_raw(0x00D0_0005);
#[doc = include_str!("name_binding.doc/DIALOG.md")]
pub const DIALOG: ComponentId = ComponentId::from_raw(0x00D0_0006);
#[doc = include_str!("name_binding.doc/TABS.md")]
pub const TABS: ComponentId = ComponentId::from_raw(0x00D0_0007);
#[doc = include_str!("name_binding.doc/LIST.md")]
pub const LIST: ComponentId = ComponentId::from_raw(0x00D0_0008);
#[doc = include_str!("name_binding.doc/TREE.md")]
pub const TREE: ComponentId = ComponentId::from_raw(0x00D0_0009);
#[doc = include_str!("name_binding.doc/TABLE.md")]
pub const TABLE: ComponentId = ComponentId::from_raw(0x00D0_000A);
#[doc = include_str!("name_binding.doc/DOCKING.md")]
pub const DOCKING: ComponentId = ComponentId::from_raw(0x00D0_000B);
#[doc = include_str!("name_binding.doc/RADIO.md")]
pub const RADIO: ComponentId = ComponentId::from_raw(0x00D0_000C);
// 2026-08-07. These two are the ids the shipped widgets carry, and they are the
// ids `Tabs` and `List` carry as well — `Progress` was numbered 0x00D0_0007 and
// `Tabs` was too, `Slider` 0x00D0_0008 and `List` too. The collision is real and
// it is not this crate's to resolve: a mirror that renumbered them would bind
// `<progress/>` to an id no shipped widget answers to, which is the exact defect
// this stage closes. It is harmless only for as long as no factory claims either
// number (a registry keyed by id would then mount a progress bar for `<Tabs/>`),
// so the widgets' own constants are where it has to be fixed, and the facade's
// drift test is what will fail here when they are.
#[doc = include_str!("name_binding.doc/PROGRESS.md")]
pub const PROGRESS: ComponentId = ComponentId::from_raw(0x00D0_0007);
#[doc = include_str!("name_binding.doc/SLIDER.md")]
pub const SLIDER: ComponentId = ComponentId::from_raw(0x00D0_0008);

#[doc = include_str!("name_binding.doc/TagPosition.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TagPosition {
    #[doc = include_str!("name_binding.doc/Element.md")]
    Element,
    #[doc = include_str!("name_binding.doc/Component.md")]
    Component,
}

#[doc = include_str!("name_binding.doc/builtin_component_id.md")]
#[must_use]
pub fn builtin_component_id(tag: &str) -> Option<ComponentId> {
    let id = match tag {
        // The §2.12 slot outlet — a position, not a component (see above).
        OUTLET_TAG => SLOT_OUTLET,
        // The controls with a **base-element** spelling as well as a component
        // one: HTML has a `<button>`, an `<input>`, a `<select>`, so this toolkit
        // has them too, and each is the same control either way — a lowercase tag
        // is not a user component that happens to be spelled that, and not a
        // `div` (which is what it used to resolve to, by falling through to
        // `CONTAINER`). Same "one handle, two spellings" rule the `Image`/`img`
        // pairs below already follow. These are exactly the controls the facade's
        // UA stylesheet styles by type, which is what made the split visible:
        // `<checkbox>` was a styled box with no behaviour while `Checkbox::new()`
        // was the control, and the one `checkbox { … }` rule reached two
        // different trees.
        "Button" | "button" => BUTTON,
        // `textarea` is a third spelling of the one control: HTML's two tags are
        // one widget here (`Input::multiline`), and *which* of them a node is
        // stays readable from its local name rather than from a second id —
        // `EditableField::is_multiline` already decides multi-line editing by
        // `local_name() == "textarea"`, so the distinction the tags carry is
        // carried, and nothing new had to be invented to carry it.
        "Input" | "input" | "textarea" => INPUT,
        "Checkbox" | "checkbox" => CHECKBOX,
        "Radio" | "radio" => RADIO,
        "Select" | "select" => SELECT,
        "Table" | "table" => TABLE,
        // The two controls that were authorable in neither spelling: they hashed
        // by POSITION into an id nothing is registered under, so `<progress/>`
        // mounted nothing and said nothing. Both get the lowercase spelling too,
        // and not on the general "HTML has one" argument — HTML has a
        // `<progress>` but nothing it spells `<slider>` — but on the narrower one
        // that decides every pair above it: each widget's own `view()` writes
        // `progress` / `slider` as its LOCAL_NAME, so a `slider { … }` rule
        // already reaches the Rust-built control. Without the lowercase tag the
        // same rule would also reach a `div` an author wrote as `<slider>`, which
        // is one selector over two different trees.
        "Progress" | "progress" => PROGRESS,
        "Slider" | "slider" => SLIDER,
        // Widgets with a component spelling only. The UA sheet styles none of
        // them by type, and a lowercase tag no rule reaches would be vocabulary
        // with nothing behind it; `Menu`, `Tabs`, `Tree`, `List` and `Docking`
        // also have no HTML element that means what they mean, and `Dialog`'s
        // does not (a `<dialog>` is a box, this is a titled widget).
        "Menu" => MENU,
        "Dialog" => DIALOG,
        "Tabs" => TABS,
        "List" => LIST,
        "Tree" => TREE,
        "Docking" => DOCKING,
        // Non-container primitives (component and element spellings).
        "Image" | "img" => IMAGE,
        "Canvas" | "canvas" => CANVAS,
        "Scroll" | "scroll" => SCROLL,
        "Portal" | "portal" => PORTAL,
        "Layer" | "layer" => LAYER,
        "ForeignSurface" => FOREIGN_SURFACE,
        // Every other tag names no built-in; what it binds to depends on the
        // position it was written in (see `component_id_for`).
        _ => return None,
    };
    Some(id)
}

#[doc = include_str!("name_binding.doc/component_id_for.md")]
#[must_use]
pub fn component_id_for(tag: &str, position: TagPosition) -> ComponentId {
    if let Some(builtin) = builtin_component_id(tag) {
        return builtin;
    }
    match position {
        TagPosition::Element => CONTAINER,
        TagPosition::Component => user_component_id(tag),
    }
}

#[doc = include_str!("name_binding.doc/local_name_for.md")]
#[must_use]
pub fn local_name_for(tag: &str, position: TagPosition) -> Option<&str> {
    if position != TagPosition::Element || tag == OUTLET_TAG {
        return None;
    }
    Some(tag)
}

#[doc = include_str!("name_binding.doc/user_component_id.md")]
#[must_use]
pub fn user_component_id(tag: &str) -> ComponentId {
    ComponentId::from_raw(hash_name_64(tag) | 0x8000_0000_0000_0000)
}

#[doc = include_str!("name_binding.doc/attr_symbol_on.md")]
#[must_use]
pub fn attr_symbol_on(tag: &str, name: &str) -> InternedSymbol {
    if tag == OUTLET_TAG && name == "name" {
        return SLOT;
    }
    if let Some(symbol) = control_attr_symbol(tag, name) {
        return symbol;
    }
    attr_symbol(name)
}

#[doc = include_str!("name_binding.doc/control_attr_symbol.md")]
#[must_use]
fn control_attr_symbol(tag: &str, name: &str) -> Option<InternedSymbol> {
    let symbol = match (tag, name) {
        ("Slider" | "slider", "min") => SLIDER_MIN,
        ("Slider" | "slider", "max") => SLIDER_MAX,
        ("Slider" | "slider", "step") => SLIDER_STEP,
        // The low end, because a single slider is a range of one point and the
        // high end defaults to it. `<slider low=… high=…>` is deliberately absent:
        // a two-thumb slider needs a third fact — `SLIDER_RANGED` — that no
        // authored attribute states, and a pair of ends without it is a control
        // whose two thumbs move together. The closed contract turns the attempt
        // into a compile error that names the attribute instead.
        ("Slider" | "slider", "value") => SLIDER_LOW,
        ("Progress" | "progress", "value") => PROGRESS_VALUE,
        _ => return None,
    };
    Some(symbol)
}

#[doc = include_str!("name_binding.doc/is_numeric_attribute_on.md")]
#[must_use]
fn is_numeric_attribute_on(tag: &str, name: &str) -> bool {
    matches!(
        (tag, name),
        ("Slider" | "slider", "min" | "max" | "step" | "value")
            | ("Progress" | "progress", "value")
    )
}

#[doc = include_str!("name_binding.doc/attr_value_on.md")]
#[must_use]
pub fn attr_value_on(tag: &str, name: &str, value: &UiIrExpression) -> UiIrExpression {
    let UiIrExpression::Literal(literal) = value else {
        return value.clone();
    };
    if !is_numeric_attribute_on(tag, name) {
        return value.clone();
    }
    match literal_number(literal) {
        Some(number) => UiIrExpression::literal(UiIrValue::Float(number)),
        None => value.clone(),
    }
}

#[doc = include_str!("name_binding.doc/literal_number.md")]
#[must_use]
fn literal_number(value: &UiIrValue) -> Option<f64> {
    let number = match value {
        #[allow(clippy::cast_precision_loss)]
        UiIrValue::Int(whole) => *whole as f64,
        UiIrValue::Float(number) => *number,
        UiIrValue::Text(text) => text.trim().parse().ok()?,
        UiIrValue::Null | UiIrValue::Bool(_) => return None,
    };
    number.is_finite().then_some(number)
}

#[doc = include_str!("name_binding.doc/attr_symbol.md")]
#[must_use]
pub fn attr_symbol(name: &str) -> InternedSymbol {
    match name {
        "class" => CLASS,
        "id" => ID,
        // A declaration block written on the element, admitted at cascade origin
        // `Inline` (§4.14). It has to be a reserved symbol rather than a hashed
        // one for the same reason `placeholder` did: `StyleAttributes::extract`
        // reads this symbol and no other, so a hashed `style` prop is carried
        // through the whole tree and then never compiled.
        "style" => STYLE,
        "type" => TYPE,
        "src" => SRC,
        "disabled" => DISABLED,
        "checked" => CHECKED,
        // The §4.16 placeholder hint. It is a *synthetic part* rather than a
        // selector input: the runtime paints it straight off this symbol while the
        // field's value is empty (`GeneratedContent::placeholder`, resolved through
        // `input::part(placeholder)`), and it is never a tree node. A hashed
        // `placeholder` prop reaches none of that — which is why an authored
        // `<input placeholder="…">` used to render no hint at all while
        // `Input::placeholder(…)` rendered one.
        "placeholder" => PLACEHOLDER,
        // A stable testing identifier, not a style attribute (spec §7.5).
        "test-id" => TEST_ID,
        "automation-id" => AUTOMATION_ID,
        // A §2.12 slot assignment — structure the runtime routes on, not a
        // selector input.
        "slot" => SLOT,
        other => fallback_symbol(other),
    }
}

#[doc = include_str!("name_binding.doc/event_symbol.md")]
#[must_use]
pub fn event_symbol(name: &str) -> InternedSymbol {
    match name {
        "click" => EVENT_CLICK,
        "input" => EVENT_INPUT,
        "change" => EVENT_CHANGE,
        // Spelled without a separator to match `click`/`input`/`change`: these
        // are markup attribute names (`on:longpress=…`), and one convention
        // across all of them is worth more than matching any outside spelling.
        "longpress" => EVENT_LONG_PRESS,
        "keydown" => EVENT_KEY_DOWN,
        "keyup" => EVENT_KEY_UP,
        // Same no-separator convention as `keydown`/`longpress`, and the same
        // spelling the DOM uses for the same four events.
        "pointerdown" => EVENT_POINTER_DOWN,
        "pointerup" => EVENT_POINTER_UP,
        "pointermove" => EVENT_POINTER_MOVE,
        "pointercancel" => EVENT_POINTER_CANCEL,
        // The crossings. The DOM's bubbling spellings — `pointerover` and
        // `pointerout` — are deliberately absent rather than aliased onto these:
        // a command routes either way here, so the two DOM pairs would be one
        // event under two names, and a name that means what another name means
        // is how an author learns the difference is real when it is not.
        "pointerenter" => EVENT_POINTER_ENTER,
        "pointerleave" => EVENT_POINTER_LEAVE,
        // The wheel, spelled the way the DOM spells it — and it is one word
        // either way, so the no-separator convention above costs nothing here.
        // Not `scroll`: this fires *before* the scroll and can refuse it, so
        // naming it after the thing it prevents would be exactly backwards.
        "wheel" => EVENT_WHEEL,
        // The focus pair, spelled the way the DOM spells it and one word either
        // way. The DOM's second pair — `focusin` / `focusout` — is deliberately
        // absent: this binding already resolves through the ancestor chain, so
        // `focus` here *is* what the DOM calls `focusin`, and a second spelling
        // would be a second name for one event.
        "focus" => EVENT_FOCUS,
        "blur" => EVENT_BLUR,
        // One word in the DOM too, and the same no-separator convention. There
        // is no `menu` alias: this names the *request* for a menu, which the
        // runtime does not itself open, so a name that promised an opened menu
        // would promise something nothing does.
        "contextmenu" => EVENT_CONTEXT_MENU,
        other => fallback_symbol(other),
    }
}

#[doc = include_str!("name_binding.doc/is_reserved_event.md")]
#[must_use]
pub fn is_reserved_event(name: &str) -> bool {
    RESERVED_EVENTS.contains(&event_symbol(name))
}

#[doc = include_str!("name_binding.doc/fallback_symbol.md")]
#[must_use]
pub fn fallback_symbol(name: &str) -> InternedSymbol {
    InternedSymbol::from_raw(hash_name_32(name) | 0x8000_0000)
}

#[doc = include_str!("name_binding.doc/hash_name_32.md")]
#[must_use]
fn hash_name_32(name: &str) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    for byte in name.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

#[doc = include_str!("name_binding.doc/hash_name_64.md")]
#[must_use]
fn hash_name_64(name: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in name.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_widget_tags_map_to_their_component_ids() {
        assert_eq!(component_id_for("Button", TagPosition::Component), BUTTON);
        assert_eq!(component_id_for("Input", TagPosition::Component), INPUT);
        assert_eq!(
            component_id_for("Checkbox", TagPosition::Component),
            CHECKBOX
        );
        assert_eq!(component_id_for("Select", TagPosition::Component), SELECT);
    }

    #[test]
    fn a_builtin_wins_in_either_position() {
        // The built-in match runs before the fallback, so no position can turn a
        // shipped widget into a box or into a name hash.
        assert_eq!(component_id_for("Button", TagPosition::Element), BUTTON);
        assert_eq!(component_id_for("Button", TagPosition::Component), BUTTON);
        assert_eq!(builtin_component_id("Button"), Some(BUTTON));
        assert_eq!(builtin_component_id("Card"), None);
    }

    #[test]
    fn the_lowercase_button_tag_is_the_same_control_as_the_pascalcase_one() {
        // HTML has a `<button>`; so does this. The lowercase spelling used to name
        // no built-in at all, so it fell through to CONTAINER — an authored
        // `<button>` was a `div`.
        assert_eq!(builtin_component_id("button"), Some(BUTTON));
        assert_eq!(component_id_for("button", TagPosition::Element), BUTTON);
        assert_eq!(
            component_id_for("button", TagPosition::Element),
            component_id_for("Button", TagPosition::Component)
        );
        assert_ne!(component_id_for("button", TagPosition::Element), CONTAINER);
    }

    #[test]
    fn every_control_the_ua_sheet_styles_has_both_spellings() {
        // The other half of "one handle, two spellings". These five widgets used to
        // answer to a PascalCase tag only, so `<checkbox>` was a styled `div` with
        // no behaviour while `Checkbox::new()` was the control — one CSS type over
        // two different trees. Each pair is now one id.
        for (element, component, id) in [
            ("button", "Button", BUTTON),
            ("input", "Input", INPUT),
            ("checkbox", "Checkbox", CHECKBOX),
            ("radio", "Radio", RADIO),
            ("select", "Select", SELECT),
            ("table", "Table", TABLE),
        ] {
            assert_eq!(builtin_component_id(element), Some(id), "{element}");
            assert_eq!(
                component_id_for(element, TagPosition::Element),
                component_id_for(component, TagPosition::Component),
                "{element}/{component}"
            );
            assert_ne!(
                component_id_for(element, TagPosition::Element),
                CONTAINER,
                "{element}"
            );
        }
    }

    #[test]
    fn textarea_is_the_input_widget_and_says_so_in_its_local_name() {
        // HTML's two text-entry tags are one widget here, so `<textarea>` binds to
        // the *same* id as `<input>` — and the distinction survives in the local
        // name, which is what `EditableField::is_multiline` reads. No second
        // component id, and no PascalCase `Textarea` invented to hold one.
        assert_eq!(builtin_component_id("textarea"), Some(INPUT));
        assert_eq!(
            component_id_for("textarea", TagPosition::Element),
            component_id_for("input", TagPosition::Element)
        );
        assert_eq!(
            local_name_for("textarea", TagPosition::Element),
            Some("textarea")
        );
        assert_eq!(local_name_for("input", TagPosition::Element), Some("input"));
    }

    #[test]
    fn the_widgets_the_ua_sheet_does_not_style_keep_one_spelling() {
        // The rule has a boundary, and it is stated rather than implied: a
        // lowercase spelling exists where a UA rule reaches it. Nothing here is a
        // reserved element tag, so `<menu>` and `<dialog>` stay ordinary boxes an
        // author may name.
        for tag in ["menu", "dialog", "tabs", "list", "tree", "docking"] {
            assert_eq!(builtin_component_id(tag), None, "{tag}");
            assert_eq!(
                component_id_for(tag, TagPosition::Element),
                CONTAINER,
                "{tag}"
            );
        }
    }

    #[test]
    fn the_two_controls_that_were_authorable_in_no_spelling_now_are() {
        // THE defect. Neither tag named a built-in, so `<progress/>` fell through
        // to `component_id_for`, which hashed it BY POSITION into an id nobody
        // registered — an element that mounts nothing, with no diagnostic. Both
        // spellings bind to the shipped widget now, and to the same one.
        assert_eq!(builtin_component_id("progress"), Some(PROGRESS));
        assert_eq!(builtin_component_id("slider"), Some(SLIDER));
        for (element, component, id) in [
            ("progress", "Progress", PROGRESS),
            ("slider", "Slider", SLIDER),
        ] {
            assert_eq!(
                component_id_for(element, TagPosition::Element),
                component_id_for(component, TagPosition::Component),
                "{element}/{component}"
            );
            assert_eq!(component_id_for(element, TagPosition::Element), id);
            assert_ne!(
                component_id_for(element, TagPosition::Element),
                CONTAINER,
                "{element}"
            );
            assert_ne!(
                component_id_for(element, TagPosition::Element),
                user_component_id(element),
                "{element} must not hash into the user namespace"
            );
            // And the tag still contributes the local name the widget's own
            // `view()` writes, so one `slider { … }` rule reaches both trees.
            assert_eq!(local_name_for(element, TagPosition::Element), Some(element));
        }
    }

    #[test]
    fn a_sliders_scale_binds_to_the_symbols_the_tiers_below_read() {
        // The scale is read by `SliderScale::read` and by the runtime's drag,
        // neither of which can call this module — so a hashed `min` is carried
        // through the whole tree and then read by nobody.
        assert_eq!(attr_symbol_on("slider", "min"), SLIDER_MIN);
        assert_eq!(attr_symbol_on("slider", "max"), SLIDER_MAX);
        assert_eq!(attr_symbol_on("slider", "step"), SLIDER_STEP);
        assert_eq!(attr_symbol_on("slider", "value"), SLIDER_LOW);
        for name in ["min", "max", "step", "value"] {
            assert_ne!(
                attr_symbol_on("slider", name),
                fallback_symbol(name),
                "{name}"
            );
            // Both spellings of the tag, or the component form would be a
            // different control from the element one.
            assert_eq!(
                attr_symbol_on("Slider", name),
                attr_symbol_on("slider", name),
                "{name}"
            );
        }
    }

    #[test]
    fn value_still_means_the_fields_text_everywhere_but_a_control() {
        // The binding that could not be tag-blind. `value` on an `<input>` is the
        // field's DEFAULT text and hashes on purpose — the reserved
        // `obscur_view::view::VALUE` declares a *controlled* value instead, and
        // `InputProps::VALUE` is the hash. One tag-blind table would have made
        // every authored field's value a slider position.
        assert_eq!(attr_symbol_on("input", "value"), fallback_symbol("value"));
        assert_eq!(attr_symbol_on("div", "value"), fallback_symbol("value"));
        assert_eq!(attr_symbol_on("Card", "value"), fallback_symbol("value"));
        assert_ne!(attr_symbol_on("slider", "value"), fallback_symbol("value"));
        assert_ne!(
            attr_symbol_on("progress", "value"),
            attr_symbol_on("slider", "value"),
            "two controls, two meanings of the same word"
        );
        // And nothing else on a control tag is rebound: only the named pairs are.
        assert_eq!(attr_symbol_on("slider", "class"), CLASS);
        assert_eq!(
            attr_symbol_on("slider", "min-width"),
            attr_symbol("min-width")
        );
        assert_eq!(attr_symbol_on("progress", "min"), fallback_symbol("min"));
    }

    #[test]
    fn a_progress_value_binds_to_the_widgets_own_symbol() {
        assert_eq!(attr_symbol_on("progress", "value"), PROGRESS_VALUE);
        assert_eq!(attr_symbol_on("Progress", "value"), PROGRESS_VALUE);
        // Pinned here as well as in the facade's drift test, because this crate
        // restates a constant that lives in `obscur_widgets::Progress`.
        assert_eq!(PROGRESS_VALUE.raw(), 0x00D1_0702);
        assert_eq!(PROGRESS_VALUE.raw() & 0x8000_0000, 0);
    }

    #[test]
    fn a_quoted_number_on_a_control_lowers_to_a_number() {
        // Markup spells its values with quotes, and the tiers that read a scale
        // accept `Int` and `Float` and nothing else — so `Text("0")` reaches them
        // as no value at all and the control keeps the default `0..1`. The
        // attribute would compile, travel the tree, and mean nothing.
        let quoted = UiIrExpression::literal(UiIrValue::Text("0.5".into()));
        assert_eq!(
            attr_value_on("progress", "value", &quoted),
            UiIrExpression::literal(UiIrValue::Float(0.5))
        );
        // Both authored spellings converge on one value, so `min="0"` and
        // `min={0}` build the same node — and the same one `Slider::over` writes.
        for spelling in [
            UiIrExpression::literal(UiIrValue::Text("10".into())),
            UiIrExpression::literal(UiIrValue::Int(10)),
            UiIrExpression::literal(UiIrValue::Float(10.0)),
        ] {
            assert_eq!(
                attr_value_on("slider", "max", &spelling),
                UiIrExpression::literal(UiIrValue::Float(10.0)),
                "{spelling:?}"
            );
        }
    }

    #[test]
    fn a_value_that_names_no_number_is_left_exactly_as_written() {
        // A guess would be worse than the reader's own default, and the value
        // stays visible in the tree for whoever is asking why.
        for text in ["wide", "", "inf", "NaN"] {
            let written = UiIrExpression::literal(UiIrValue::Text(text.into()));
            assert_eq!(attr_value_on("slider", "min", &written), written, "{text}");
        }
        // A binding has no compile-time value to normalise at all.
        let bound = UiIrExpression::binding("state.volume");
        assert_eq!(attr_value_on("slider", "value", &bound), bound);
    }

    #[test]
    fn the_value_rule_reaches_no_attribute_but_a_controls_own() {
        // The disjoint half: this is a rebinding of named pairs, not a rule about
        // numbers. A `<div count="3">` keeps its text, or every authored
        // attribute in the language would silently change type.
        let quoted = UiIrExpression::literal(UiIrValue::Text("3".into()));
        for (tag, name) in [
            ("div", "count"),
            ("input", "value"),
            ("slider", "width"),
            ("progress", "min"),
        ] {
            assert_eq!(attr_value_on(tag, name, &quoted), quoted, "{tag}/{name}");
        }
    }

    #[test]
    fn every_reserved_control_attribute_is_one_the_tiers_below_read_as_a_number() {
        // The two tables are separate on purpose — "which symbol" and "is it a
        // number" are different questions — so pin that they agree on today's
        // set. A control attribute that is a string will need this list split,
        // and this is where that shows up.
        for (tag, name) in [
            ("slider", "min"),
            ("slider", "max"),
            ("slider", "step"),
            ("slider", "value"),
            ("progress", "value"),
        ] {
            assert!(control_attr_symbol(tag, name).is_some(), "{tag}/{name}");
            assert!(is_numeric_attribute_on(tag, name), "{tag}/{name}");
        }
    }

    #[test]
    fn image_element_and_component_spellings_agree() {
        assert_eq!(component_id_for("img", TagPosition::Element), IMAGE);
        assert_eq!(component_id_for("Image", TagPosition::Component), IMAGE);
    }

    #[test]
    fn generic_and_unknown_element_tags_fall_back_to_container() {
        for tag in ["div", "span", "header", "app", "something-custom"] {
            assert_eq!(component_id_for(tag, TagPosition::Element), CONTAINER);
        }
    }

    #[test]
    fn an_unrecognised_component_tag_is_its_own_id_not_a_box() {
        // The regression this position argument exists for: `<Card/>` used to be
        // a `div`. It is now Card.
        assert_eq!(
            component_id_for("Card", TagPosition::Component),
            user_component_id("Card")
        );
        assert_ne!(component_id_for("Card", TagPosition::Component), CONTAINER);
        // And two different components are two different ids — a hash that
        // collapsed them would be no better than CONTAINER.
        assert_ne!(user_component_id("Card"), user_component_id("Panel"));
    }

    #[test]
    fn user_component_ids_live_outside_the_builtin_range() {
        // Disjoint by construction, not by luck: a user id carries the top bit
        // and every built-in handle is a small low-range constant.
        for tag in ["Card", "Panel", "MyVeryLongComponentName", ""] {
            assert!(user_component_id(tag).raw() & 0x8000_0000_0000_0000 != 0);
        }
        for builtin in [
            CONTAINER,
            IMAGE,
            SCROLL,
            CANVAS,
            PORTAL,
            LAYER,
            FOREIGN_SURFACE,
            BUTTON,
            INPUT,
            CHECKBOX,
            SELECT,
            MENU,
            DIALOG,
            TABS,
            LIST,
            TREE,
            TABLE,
            DOCKING,
            RADIO,
        ] {
            assert_eq!(builtin.raw() & 0x8000_0000_0000_0000, 0);
        }
    }

    #[test]
    fn user_component_ids_are_deterministic() {
        assert_eq!(user_component_id("Card"), user_component_id("Card"));
    }

    #[test]
    fn an_element_tag_is_its_own_local_name() {
        // The defect this function exists for: an authored element reached the tree
        // with a ComponentId and nothing else, so no type selector could match it —
        // `button { … }`, and the whole UA stylesheet with it, styled nothing an
        // author wrote.
        for tag in ["button", "div", "span", "td", "img", "something-custom"] {
            assert_eq!(
                local_name_for(tag, TagPosition::Element),
                Some(tag),
                "{tag}"
            );
        }
    }

    #[test]
    fn a_component_tag_has_no_local_name() {
        // A component names a component; the elements its render produces carry
        // their own names. And a type selector matches ASCII-case-insensitively, so
        // a local name `Card` would be indistinguishable from a `<card>` element.
        assert_eq!(local_name_for("Card", TagPosition::Component), None);
        assert_eq!(local_name_for("Button", TagPosition::Component), None);
    }

    #[test]
    fn the_local_name_symbol_is_not_the_authored_type_symbol() {
        // The two channels HTML keeps apart, and this engine now does too: the tag
        // says *what a thing is*, `type` says *which kind*. They shared one symbol,
        // and the price was that `<button type="submit">` had CSS type `submit` and
        // so matched no `button { … }` rule at all.
        assert_ne!(LOCAL_NAME, TYPE);
        assert_eq!(attr_symbol("type"), TYPE);
        assert_ne!(attr_symbol("type"), LOCAL_NAME);
        // No attribute name an author can write binds to LOCAL_NAME — it is the
        // element's identity, contributed by the lowering paths alone.
        for name in ["local-name", "localname", "tag", "type"] {
            assert_ne!(attr_symbol(name), LOCAL_NAME, "{name}");
        }
    }

    #[test]
    fn an_authored_type_attribute_leaves_the_local_name_alone() {
        // The defect this stage closed: an authored `type` used to make the tag
        // contribute *no* name. It is a separate symbol now, so the tag always
        // contributes, whatever else the author wrote.
        assert_eq!(
            local_name_for("button", TagPosition::Element),
            Some("button")
        );
        assert_eq!(attr_symbol_on("button", "type"), TYPE);
        assert_ne!(attr_symbol_on("button", "type"), LOCAL_NAME);
    }

    #[test]
    fn the_outlet_tag_has_no_local_name_either() {
        // A §2.12 outlet is a position marker that projection substitutes away, so
        // no retained node ever carries `slot` as a type — and an authored
        // `<slot name="header"/>` has to stay *the same node* a component's own
        // `ViewNode::slot("header")` builds. A property on one spelling and not the
        // other would be two outlets.
        assert_eq!(local_name_for(OUTLET_TAG, TagPosition::Element), None);
    }

    #[test]
    fn reserved_attributes_map_to_reserved_symbols() {
        assert_eq!(attr_symbol("class"), CLASS);
        assert_eq!(attr_symbol("id"), ID);
        assert_eq!(attr_symbol("type"), TYPE);
        assert_eq!(attr_symbol("src"), SRC);
        assert_eq!(attr_symbol("disabled"), DISABLED);
        assert_eq!(attr_symbol("checked"), CHECKED);
        assert_eq!(attr_symbol("placeholder"), PLACEHOLDER);
        assert_eq!(attr_symbol("test-id"), TEST_ID);
        assert_eq!(attr_symbol("automation-id"), AUTOMATION_ID);
        assert_eq!(attr_symbol("slot"), SLOT);
    }

    #[test]
    fn the_placeholder_hint_is_reserved_because_only_the_painter_reads_it() {
        // The one reserved attribute here that is not a selector input. §4.16 makes
        // the placeholder a synthetic part the runtime generates from this symbol
        // and this symbol alone (`StyleAttributes::placeholder` →
        // `GeneratedContent::placeholder`), so a hashed spelling reaches nothing at
        // all — which is what an authored `<input placeholder="…">` used to do
        // while `Input::placeholder(…)` painted a hint. Same attribute, two front
        // ends, one visible outcome and one invisible one.
        assert_eq!(attr_symbol("placeholder"), PLACEHOLDER);
        assert_ne!(attr_symbol("placeholder"), fallback_symbol("placeholder"));
        // It is also the symbol `obscur-view` reserves for it; the two crates
        // restate the table and this is the pin on this side.
        assert_eq!(PLACEHOLDER.raw(), 0x0000_0009);
    }

    #[test]
    fn the_outlet_tag_binds_to_the_reserved_outlet_id() {
        // `<slot/>` used to be an unknown element tag — the generic CONTAINER box,
        // indistinguishable from a `<div>`. It is now the §2.12 outlet.
        assert_eq!(builtin_component_id(OUTLET_TAG), Some(SLOT_OUTLET));
        assert_eq!(component_id_for("slot", TagPosition::Element), SLOT_OUTLET);
        assert_ne!(component_id_for("slot", TagPosition::Element), CONTAINER);
    }

    #[test]
    fn the_outlet_id_aliases_no_other_namespace() {
        // Disjoint by construction: every primitive is `0x00C0_xxxx`, every widget
        // `0x00D0_xxxx`, and every user component carries the top bit. Mirrors
        // `obscur_view::view::slot`'s own assertion; the facade's drift guard is
        // what proves the two constants are the same value.
        assert_eq!(SLOT_OUTLET.raw() & 0x8000_0000_0000_0000, 0);
        assert!(SLOT_OUTLET.raw() < 0x00C0_0000);
        assert_ne!(SLOT_OUTLET, CONTAINER);
    }

    #[test]
    fn name_binds_to_the_slot_symbol_only_on_the_outlet_tag() {
        // THE binding the audit called impossible. `name` is the outlet's slot on
        // `<slot>`, and an ordinary hashed prop everywhere else — the two meanings
        // the tag-blind `attr_symbol` was said to be unable to hold apart.
        assert_eq!(attr_symbol_on("slot", "name"), SLOT);
        assert_eq!(attr_symbol_on("div", "name"), fallback_symbol("name"));
        assert_eq!(attr_symbol_on("Card", "name"), fallback_symbol("name"));
        assert_ne!(attr_symbol_on("div", "name"), SLOT);
    }

    #[test]
    fn an_outlets_slot_and_a_childs_slot_are_the_same_symbol() {
        // They are the two halves of one join: `project_slots` reads an outlet's
        // name and a child's assignment through the same `slot_assignment`. Two
        // symbols here would be two names that never meet.
        assert_eq!(attr_symbol_on("slot", "name"), attr_symbol("slot"));
    }

    #[test]
    fn only_the_name_attribute_is_special_on_the_outlet_tag() {
        // The disjoint half: the rebinding is the pair (`slot`, `name`), not the
        // tag. Everything else on a `<slot>` binds exactly as it would anywhere.
        assert_eq!(attr_symbol_on("slot", "class"), CLASS);
        assert_eq!(attr_symbol_on("slot", "data-x"), fallback_symbol("data-x"));
        assert_eq!(attr_symbol_on("slot", "slot"), SLOT);
    }

    #[test]
    fn attr_symbol_on_is_attr_symbol_off_the_outlet_tag() {
        // The tagless entry point is this one's non-outlet branch, so no caller can
        // be reached by only one of them and get a different answer.
        for name in ["class", "id", "type", "src", "disabled", "checked", "zzz"] {
            assert_eq!(attr_symbol_on("div", name), attr_symbol(name), "{name}");
            assert_eq!(attr_symbol_on("slot", name), attr_symbol(name), "{name}");
        }
    }

    #[test]
    fn the_slot_assignment_is_reserved_rather_than_hashed() {
        // It has to be: the runtime reads it (to keep a component host's authored
        // children and match them to the outlets its render declares), and the
        // runtime cannot call this function. A hashed value would agree only
        // between the two front ends.
        assert_ne!(attr_symbol("slot"), fallback_symbol("slot"));
        assert_eq!(SLOT.raw() & 0x8000_0000, 0);
    }

    #[test]
    fn reserved_events_map_to_reserved_symbols() {
        assert_eq!(event_symbol("click"), EVENT_CLICK);
        assert_eq!(event_symbol("input"), EVENT_INPUT);
        assert_eq!(event_symbol("change"), EVENT_CHANGE);
    }

    #[test]
    fn the_raw_pointer_names_lower_to_their_reserved_symbols() {
        // The defect this campaign closes, one stage at a time: a name the
        // runtime dispatches but the front end hashes is a binding that compiles
        // and never fires. Each of these has a real emitter in `handle_pointer`.
        assert_eq!(event_symbol("pointerdown"), EVENT_POINTER_DOWN);
        assert_eq!(event_symbol("pointerup"), EVENT_POINTER_UP);
        assert_eq!(event_symbol("pointermove"), EVENT_POINTER_MOVE);
        assert_eq!(event_symbol("pointercancel"), EVENT_POINTER_CANCEL);
        // 2026-08-03. `pointerenter` and `pointerleave` were in the *unreserved*
        // list below, on the rule that a name with no emitter must stay
        // unreserved. They have one now — `handle_pointer` dispatches them from
        // the hover transition — so they move up here. The rule did not change;
        // what changed is which side of it these two are on, and this test
        // asserts both sides.
        assert_eq!(event_symbol("pointerenter"), EVENT_POINTER_ENTER);
        assert_eq!(event_symbol("pointerleave"), EVENT_POINTER_LEAVE);
        for name in [
            "pointerdown",
            "pointerup",
            "pointermove",
            "pointercancel",
            "pointerenter",
            "pointerleave",
        ] {
            assert!(is_reserved_event(name), "{name}");
            assert_ne!(event_symbol(name), fallback_symbol(name), "{name}");
        }
        // And the names the runtime still does not emit stay unreserved, so a
        // front end reports them rather than blessing them. `pointerover` and
        // `pointerout` are here on purpose and not for want of an emitter: a
        // command routes either way in this toolkit, so the DOM's bubbling pair
        // would be a second name for `pointerenter` / `pointerleave`.
        //
        // 2026-08-03. `wheel` used to be in this list, on the same rule the two
        // crossings moved under: a name with no emitter must stay unreserved. It
        // has one now — `handle_scroll` dispatches it before the scroll it can
        // refuse — so it moved to its own test below. `contextmenu` followed it
        // out of this list the same day and for the same reason, once the winit
        // runner started delivering the secondary button that emits it; it has
        // its own test below too. The rule did not change — only which side of
        // it each name is on — and both sides are asserted.
        for name in ["pointerover", "pointerout"] {
            assert!(!is_reserved_event(name), "{name}");
        }
    }

    #[test]
    fn the_context_menu_name_lowers_to_its_reserved_symbol() {
        // Its own test rather than a line in the pointer one: this is not a
        // pointer *report*. It names a request the secondary button makes, and
        // the runtime emits it from that press alone — there is no context-menu
        // key and no long-press route to it, so a second emitter would have to be
        // invented before either could claim this name.
        assert_eq!(event_symbol("contextmenu"), EVENT_CONTEXT_MENU);
        assert!(is_reserved_event("contextmenu"));
        assert_ne!(
            event_symbol("contextmenu"),
            fallback_symbol("contextmenu"),
            "a hashed name would compile and never fire"
        );
        // And no alias: `menu` would promise an opened menu, which nothing here
        // opens.
        assert!(!is_reserved_event("menu"));
    }

    #[test]
    fn the_focus_pair_lowers_to_its_reserved_symbols() {
        // Its own test rather than a line in the pointer one: focus is not a
        // pointer report. A press is what moves it today, but the event names the
        // node that gained or lost focus, and the runtime emits both from the one
        // place focus changes — so any future mover emits them too.
        assert_eq!(event_symbol("focus"), EVENT_FOCUS);
        assert_eq!(event_symbol("blur"), EVENT_BLUR);
        for name in ["focus", "blur"] {
            assert!(is_reserved_event(name), "{name}");
            assert_ne!(event_symbol(name), fallback_symbol(name), "{name}");
        }
        // The DOM's second pair is deliberately NOT reserved: `focus` here is
        // resolved through the ancestor chain, so it already *is* `focusin`, and
        // a second spelling would be a second name for one event. They hash like
        // any other author name.
        for absent in ["focusin", "focusout"] {
            assert!(!is_reserved_event(absent), "{absent}");
        }
    }

    #[test]
    fn the_wheel_name_lowers_to_its_reserved_symbol() {
        // Its own test rather than a line in the pointer one: a wheel is not a
        // `PointerEventKind` and is dispatched from `handle_scroll`, not from
        // `handle_pointer`.
        assert_eq!(event_symbol("wheel"), EVENT_WHEEL);
        assert!(is_reserved_event("wheel"));
        assert_ne!(event_symbol("wheel"), fallback_symbol("wheel"));
        // `scroll` is deliberately NOT an alias: this event fires before the
        // scroll and can refuse it, and there is no separate event for the
        // scroll having happened.
        assert!(!is_reserved_event("scroll"));
    }

    #[test]
    fn unreserved_names_hash_into_the_high_namespace() {
        // The top bit is set, so a hashed name never collides with a reserved
        // low-range symbol.
        assert_ne!(attr_symbol("data-foo"), CLASS);
        assert!(attr_symbol("data-foo").raw() & 0x8000_0000 != 0);
        assert!(event_symbol("hover").raw() & 0x8000_0000 != 0);
    }

    #[test]
    fn hashing_is_deterministic() {
        assert_eq!(attr_symbol("title"), attr_symbol("title"));
        assert_eq!(event_symbol("submit"), event_symbol("submit"));
    }

    #[test]
    fn named_symbol_mirrors_fallback_symbol() {
        // `InternedSymbol::named` (foundation) is the author-facing way to name a
        // command or list, and it must resolve to the *same* handle this crate
        // lowers an unreserved `on:click="save"` / `class="…"`-style name to — so a
        // handler registered under `named("save")` is the one an `.okui`
        // `on:click="save"` reaches. Drift here silently unbinds markup from code.
        for name in ["save", "goParent", "selectUp", "increment", "data-x", ""] {
            assert_eq!(fallback_symbol(name), InternedSymbol::named(name), "{name}");
        }
    }

    #[test]
    fn reserved_symbols_are_outside_the_hash_namespace() {
        // Every reserved symbol lives in the low range (top bit clear), so the
        // fallback hash can never alias one.
        for symbol in [
            CLASS,
            ID,
            TYPE,
            LOCAL_NAME,
            SRC,
            DISABLED,
            CHECKED,
            PLACEHOLDER,
            EVENT_CLICK,
            EVENT_INPUT,
            EVENT_CHANGE,
            TEST_ID,
            AUTOMATION_ID,
            SLOT,
            SLIDER_MIN,
            SLIDER_MAX,
            SLIDER_STEP,
            SLIDER_LOW,
            PROGRESS_VALUE,
        ] {
            assert_eq!(symbol.raw() & 0x8000_0000, 0);
        }
    }
}
