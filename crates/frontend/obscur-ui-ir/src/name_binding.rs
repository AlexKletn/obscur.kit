//! The frontend name-binding registry: the single policy that maps authored
//! `.okui`/DSL **names** onto the engine's real handles.
//!
//! A `.okui` document (or the TSX-like DSL) names things with strings — a tag
//! like `Button` or `div`, an attribute like `class` or `src`, an event like
//! `click`. Both frontend lowering paths must agree on what those names bind to:
//!
//! - the **runtime bridge** ([`crate::document`] → `obscur-view` `View`), which
//!   interprets the IR into a desired tree at runtime, and
//! - the **code generator**, which emits ordinary Rust that builds the same
//!   `View` at *compile* time.
//!
//! Centralising the policy here keeps the two paths byte-for-byte consistent and
//! removes the duplicated ad-hoc hashing they used to carry.
//!
//! # Mapping
//!
//! - **Tags → [`ComponentId`]**: the built-in widgets and non-container
//!   primitives map to their stable `COMPONENT_ID`; every other tag (`div`,
//!   `span`, `header`, an app-defined element) maps to the generic
//!   [`Container`](CONTAINER) primitive, so it is a real, styleable box rather
//!   than an opaque hash.
//! - **Attribute names → [`InternedSymbol`]**: the CSS/reserved attributes
//!   (`class`, `id`, `type`, `src`, `disabled`, `checked`) map to the reserved
//!   `obscur-view` symbols the reconciler and matcher read; any other name is
//!   deterministically hashed into a high symbol namespace (top bit set) that
//!   never collides with the reserved low range.
//! - **Event names → [`InternedSymbol`]**: `click`/`input`/`change` map to the
//!   reserved `EVENT_*` symbols the runtime's dispatcher resolves; any other name
//!   is hashed like an attribute.
//!
//! # Drift
//!
//! The raw handle values below are duplicated from their canonical definitions
//! (`obscur-view` reserved symbols; the widget/primitive `COMPONENT_ID`
//! constants) because this frontend crate must not depend on the component tier.
//! A drift-guard test in the facade (`obscur-kit`) asserts every constant here
//! still equals its canonical source.

use obscur_identity::{ComponentId, InternedSymbol};

// --- Reserved attribute symbols (mirror `obscur_view::view::style_attribute`). ---

/// Reserved `class` attribute symbol (`obscur_view::view::CLASS`).
pub const CLASS: InternedSymbol = InternedSymbol::from_raw(0x0000_0001);
/// Reserved `id` attribute symbol (`obscur_view::view::ID`).
pub const ID: InternedSymbol = InternedSymbol::from_raw(0x0000_0002);
/// Reserved `type` attribute symbol (`obscur_view::view::TYPE`).
pub const TYPE: InternedSymbol = InternedSymbol::from_raw(0x0000_0003);
/// Reserved `src` attribute symbol (`obscur_view::view::SRC`).
pub const SRC: InternedSymbol = InternedSymbol::from_raw(0x0000_0004);
/// Reserved `disabled` attribute symbol (`obscur_view::view::DISABLED`).
pub const DISABLED: InternedSymbol = InternedSymbol::from_raw(0x0000_0005);
/// Reserved `checked` attribute symbol (`obscur_view::view::CHECKED`).
pub const CHECKED: InternedSymbol = InternedSymbol::from_raw(0x0000_0006);

/// Mirror of `obscur_view::view::TEST_ID` — a node's stable testing identifier
/// (spec §7.5).
///
/// Deliberately NOT a style attribute: it must not affect matching, specificity,
/// layout or visuals, so it is carried as an ordinary property and read only by a
/// selector resolver. It sits in its own `0x0000_02xx` namespace.
pub const TEST_ID: InternedSymbol = InternedSymbol::from_raw(0x0000_0201);

/// Mirror of `obscur_view::view::AUTOMATION_ID` — a node's stable identifier for
/// external automation and assistive tooling (spec §7.5).
///
/// Separate from [`TEST_ID`] because the two answer to different owners: a test-id
/// is the app's own and renames freely, an automation-id is a contract with
/// software that cannot be updated in the same commit.
pub const AUTOMATION_ID: InternedSymbol = InternedSymbol::from_raw(0x0000_0202);

// --- Reserved event symbols (mirror `obscur_view::view::event_name`). ---

/// Reserved `click` event symbol (`obscur_view::view::EVENT_CLICK`).
pub const EVENT_CLICK: InternedSymbol = InternedSymbol::from_raw(0x0000_0101);
/// Reserved `input` event symbol (`obscur_view::view::EVENT_INPUT`).
pub const EVENT_INPUT: InternedSymbol = InternedSymbol::from_raw(0x0000_0102);
/// Reserved `change` event symbol (`obscur_view::view::EVENT_CHANGE`).
pub const EVENT_CHANGE: InternedSymbol = InternedSymbol::from_raw(0x0000_0103);

// --- Primitive component ids (mirror `obscur_primitives::*::COMPONENT_ID`). ---

/// The generic box primitive (`obscur_primitives::Container`) — the default
/// binding for any tag without a more specific mapping.
pub const CONTAINER: ComponentId = ComponentId::from_raw(0x00C0_0001);
/// The image primitive (`obscur_primitives::Image`).
pub const IMAGE: ComponentId = ComponentId::from_raw(0x00C0_0003);
/// The scroll primitive (`obscur_primitives::Scroll`).
pub const SCROLL: ComponentId = ComponentId::from_raw(0x00C0_0004);
/// The canvas primitive (`obscur_primitives::Canvas`).
pub const CANVAS: ComponentId = ComponentId::from_raw(0x00C0_0005);
/// The portal primitive (`obscur_primitives::Portal`).
pub const PORTAL: ComponentId = ComponentId::from_raw(0x00C0_0006);
/// The layer primitive (`obscur_primitives::Layer`).
pub const LAYER: ComponentId = ComponentId::from_raw(0x00C0_0007);
/// The foreign-surface primitive (`obscur_primitives::ForeignSurface`).
pub const FOREIGN_SURFACE: ComponentId = ComponentId::from_raw(0x00C0_0008);

// --- Widget component ids (mirror `obscur_widgets::*::COMPONENT_ID`). ---

/// The button widget (`obscur_widgets::Button`).
pub const BUTTON: ComponentId = ComponentId::from_raw(0x00D0_0001);
/// The input widget (`obscur_widgets::Input`).
pub const INPUT: ComponentId = ComponentId::from_raw(0x00D0_0002);
/// The checkbox widget (`obscur_widgets::Checkbox`).
pub const CHECKBOX: ComponentId = ComponentId::from_raw(0x00D0_0003);
/// The select widget (`obscur_widgets::Select`).
pub const SELECT: ComponentId = ComponentId::from_raw(0x00D0_0004);
/// The menu widget (`obscur_widgets::Menu`).
pub const MENU: ComponentId = ComponentId::from_raw(0x00D0_0005);
/// The dialog widget (`obscur_widgets::Dialog`).
pub const DIALOG: ComponentId = ComponentId::from_raw(0x00D0_0006);
/// The tabs widget (`obscur_widgets::Tabs`).
pub const TABS: ComponentId = ComponentId::from_raw(0x00D0_0007);
/// The list widget (`obscur_widgets::List`).
pub const LIST: ComponentId = ComponentId::from_raw(0x00D0_0008);
/// The tree widget (`obscur_widgets::Tree`).
pub const TREE: ComponentId = ComponentId::from_raw(0x00D0_0009);
/// The table widget (`obscur_widgets::Table`).
pub const TABLE: ComponentId = ComponentId::from_raw(0x00D0_000A);
/// The docking widget (`obscur_widgets::Docking`).
pub const DOCKING: ComponentId = ComponentId::from_raw(0x00D0_000B);
/// The radio widget (`obscur_widgets::Radio`).
pub const RADIO: ComponentId = ComponentId::from_raw(0x00D0_000C);

/// Resolves a tag name to the [`ComponentId`] it binds to.
///
/// Built-in widgets and non-container primitives resolve to their stable
/// `COMPONENT_ID`; the lowercase `img`/`canvas`/… element spellings resolve to
/// the same primitive as their PascalCase component spelling. Every other tag —
/// `div`, `span`, `header`, an app-defined element — resolves to the generic
/// [`CONTAINER`] primitive, so an authored element is always a real, styleable
/// box.
#[must_use]
pub fn component_id_for(tag: &str) -> ComponentId {
    match tag {
        // Widgets (PascalCase component spelling).
        "Button" => BUTTON,
        "Input" => INPUT,
        "Checkbox" => CHECKBOX,
        "Select" => SELECT,
        "Menu" => MENU,
        "Dialog" => DIALOG,
        "Tabs" => TABS,
        "List" => LIST,
        "Tree" => TREE,
        "Table" => TABLE,
        "Docking" => DOCKING,
        "Radio" => RADIO,
        // Non-container primitives (component and element spellings).
        "Image" | "img" => IMAGE,
        "Canvas" | "canvas" => CANVAS,
        "Scroll" | "scroll" => SCROLL,
        "Portal" | "portal" => PORTAL,
        "Layer" | "layer" => LAYER,
        "ForeignSurface" => FOREIGN_SURFACE,
        // Every other tag is a generic styleable box.
        _ => CONTAINER,
    }
}

/// Resolves an attribute name to the [`InternedSymbol`] it binds to.
///
/// The reserved CSS/state attributes map to the `obscur-view` symbols the
/// reconciler and matcher read; any other name is hashed into the high symbol
/// namespace (see [`fallback_symbol`]).
#[must_use]
pub fn attr_symbol(name: &str) -> InternedSymbol {
    match name {
        "class" => CLASS,
        "id" => ID,
        "type" => TYPE,
        "src" => SRC,
        "disabled" => DISABLED,
        "checked" => CHECKED,
        // A stable testing identifier, not a style attribute (spec §7.5).
        "test-id" => TEST_ID,
        "automation-id" => AUTOMATION_ID,
        other => fallback_symbol(other),
    }
}

/// Resolves an event name to the [`InternedSymbol`] it binds to.
///
/// `click`/`input`/`change` map to the reserved `EVENT_*` symbols the runtime
/// dispatcher resolves; any other name is hashed like an attribute.
#[must_use]
pub fn event_symbol(name: &str) -> InternedSymbol {
    match name {
        "click" => EVENT_CLICK,
        "input" => EVENT_INPUT,
        "change" => EVENT_CHANGE,
        other => fallback_symbol(other),
    }
}

/// Hashes an unreserved name into a high symbol namespace (top bit set) that
/// never collides with the reserved low range.
///
/// This is the deterministic placeholder both frontend paths share for names
/// that have no reserved symbol; a name interner threaded through the whole
/// pipeline would replace it.
#[must_use]
pub fn fallback_symbol(name: &str) -> InternedSymbol {
    InternedSymbol::from_raw(hash_name_32(name) | 0x8000_0000)
}

/// Deterministic FNV-1a 32-bit hash of a name.
#[must_use]
fn hash_name_32(name: &str) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    for byte in name.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_widget_tags_map_to_their_component_ids() {
        assert_eq!(component_id_for("Button"), BUTTON);
        assert_eq!(component_id_for("Input"), INPUT);
        assert_eq!(component_id_for("Checkbox"), CHECKBOX);
        assert_eq!(component_id_for("Select"), SELECT);
    }

    #[test]
    fn image_element_and_component_spellings_agree() {
        assert_eq!(component_id_for("img"), IMAGE);
        assert_eq!(component_id_for("Image"), IMAGE);
    }

    #[test]
    fn generic_and_unknown_tags_fall_back_to_container() {
        assert_eq!(component_id_for("div"), CONTAINER);
        assert_eq!(component_id_for("span"), CONTAINER);
        assert_eq!(component_id_for("header"), CONTAINER);
        assert_eq!(component_id_for("app"), CONTAINER);
        assert_eq!(component_id_for("something-custom"), CONTAINER);
    }

    #[test]
    fn reserved_attributes_map_to_reserved_symbols() {
        assert_eq!(attr_symbol("class"), CLASS);
        assert_eq!(attr_symbol("id"), ID);
        assert_eq!(attr_symbol("type"), TYPE);
        assert_eq!(attr_symbol("src"), SRC);
        assert_eq!(attr_symbol("disabled"), DISABLED);
        assert_eq!(attr_symbol("checked"), CHECKED);
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
    fn reserved_symbols_are_outside_the_hash_namespace() {
        // Every reserved symbol lives in the low range (top bit clear), so the
        // fallback hash can never alias one.
        for symbol in [
            CLASS,
            ID,
            TYPE,
            SRC,
            DISABLED,
            CHECKED,
            EVENT_CLICK,
            EVENT_INPUT,
            EVENT_CHANGE,
        ] {
            assert_eq!(symbol.raw() & 0x8000_0000, 0);
        }
    }
}
