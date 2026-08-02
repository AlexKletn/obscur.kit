#![doc = include_str!("name_binding.doc/_module.md")]

use obscur_identity::{ComponentId, InternedSymbol};

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

#[doc = include_str!("name_binding.doc/TEST_ID.md")]
pub const TEST_ID: InternedSymbol = InternedSymbol::from_raw(0x0000_0201);

#[doc = include_str!("name_binding.doc/AUTOMATION_ID.md")]
pub const AUTOMATION_ID: InternedSymbol = InternedSymbol::from_raw(0x0000_0202);

#[doc = include_str!("name_binding.doc/SLOT.md")]
pub const SLOT: InternedSymbol = InternedSymbol::from_raw(0x0000_0301);

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
    attr_symbol(name)
}

#[doc = include_str!("name_binding.doc/attr_symbol.md")]
#[must_use]
pub fn attr_symbol(name: &str) -> InternedSymbol {
    match name {
        "class" => CLASS,
        "id" => ID,
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
        other => fallback_symbol(other),
    }
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
        ] {
            assert_eq!(symbol.raw() & 0x8000_0000, 0);
        }
    }
}
