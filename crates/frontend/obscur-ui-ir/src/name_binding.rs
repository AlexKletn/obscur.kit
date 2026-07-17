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
//!   primitives map to their stable `COMPONENT_ID`. What an unrecognised tag maps
//!   to depends on the [`TagPosition`] it was written in — an *element* tag
//!   (`div`, `span`, `header`) maps to the generic [`Container`](CONTAINER)
//!   primitive, so it is a real, styleable box; a *component* tag (`Card`) maps
//!   to [`user_component_id`], the id `#[derive(Contract)]` computes for the very
//!   same name.
//! - **Attribute names → [`InternedSymbol`]**: the CSS/reserved attributes
//!   (`class`, `id`, `type`, `src`, `disabled`, `checked`) map to the reserved
//!   `obscur-view` symbols the reconciler and matcher read; any other name is
//!   deterministically hashed into a high symbol namespace (top bit set) that
//!   never collides with the reserved low range. Exactly one binding depends on
//!   the tag the attribute sits on — `name` on a [`OUTLET_TAG`] — which is why
//!   [`attr_symbol_on`] exists alongside [`attr_symbol`].
//! - **Event names → [`InternedSymbol`]**: `click`/`input`/`change` map to the
//!   reserved `EVENT_*` symbols the runtime's dispatcher resolves; any other name
//!   is hashed like an attribute.
//!
//! # Why a component tag is not a box
//!
//! It used to be. [`component_id_for`] took a tag and nothing else, and resolved
//! **every** unrecognised name to [`CONTAINER`] — so `<Card/>` was not a
//! mis-checked component and not a rejected one, it was a generic div, silently.
//! §2.13's contract validation "skips any tag with no registered contract" was
//! checking nothing, because the tag was never going to *be* a component.
//!
//! The position is now an argument rather than a guess, and it is the
//! distinction both front ends already make and then threw away (the `.okui`
//! lowering's `starts_uppercase`, the macro's `is_component`, both of which
//! produce a `UiIrNodeKind::Component`). A caller must now *state* which
//! fallback it wants, so the wrong one can no longer be picked by omission —
//! though it remains perfectly spellable, and only the call sites' agreement
//! with their node's kind keeps it right.
//!
//! # The id of a component nobody here can see
//!
//! §2.14's `ComponentRegistry` is a **runtime** value holding
//! `Box<dyn ComponentFactory>` trait objects. A code generator cannot consult it:
//! `CardProps::contract()` is a function that runs when the built crate runs, and
//! codegen runs while that crate is still being compiled. So the binding does not
//! ask a registry what `Card` is — it computes the id from the **name**, exactly
//! as `#[derive(Contract)]` does at expansion time (the derive calls straight into
//! [`component_id_for`], so there is one definition rather than two that agree by
//! luck). A host that registers the derived contract's id and an author who writes
//! the tag therefore meet on the same handle without either knowing about the
//! other.
//!
//! What this does **not** do is make an unregistered tag loud. `<Crad/>` (a typo)
//! binds to `user_component_id("Crad")`, nothing is registered under it, and it
//! mounts nothing — no diagnostic. Nor is it even visibly different: no part of
//! matching, cascade, layout or paint reads a `ComponentId` (only
//! `ComponentStore::mount` does), so a typo'd component tag still renders as the
//! same box it silently rendered as before. What it can no longer do is collide
//! with [`CONTAINER`] and mount whatever a host registered there.
//!
//! Making it loud is the half this does not close. The file front end could
//! (its `Compiler` already takes a `ContractRegistry`); the `okui!` macro cannot,
//! for the reason above — so the rule cannot be stated once for both front ends,
//! and stating it for one is the drift the `<VirtualFor>`-was-`<For>` finding was
//! written against. `okui_user_component.rs`'s
//! `an_unregistered_component_tag_mounts_nothing` pins the residue.
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

/// Mirror of `obscur_view::view::SLOT` — a child's `slot="…"` assignment, naming
/// the slot of the enclosing component it is projected into (spec §2.12).
///
/// Reserved rather than hashed because the runtime reads it: the reconciler keeps
/// the authored children of a node that carries one, and `project_slots` matches
/// them against the outlets the component's own render declares. A hashed symbol
/// would have made this a name only the front ends agreed on, and the tier that
/// has to act on it lives below them and cannot call this function.
///
/// Not a style attribute: a slot assignment is structure, so it must not affect
/// matching or specificity. It has a namespace of its own (`0x0000_03xx`).
pub const SLOT: InternedSymbol = InternedSymbol::from_raw(0x0000_0301);

/// Mirror of `obscur_view::view::SLOT_OUTLET` — the [`ComponentId`] of a §2.12
/// **slot outlet**, the position a component's render names for its authored
/// children to be projected into.
///
/// This is what [`OUTLET_TAG`] binds to, and it is the reason the outlet needed no
/// new syntax on either front end: `<slot name="header"/>` is an ordinary element
/// tag with an ordinary attribute, and the two [`name_binding`](self) rules
/// (`slot` → this id, and `name` → [`SLOT`] *on this tag*) are the whole feature.
///
/// It sits in a reserved range of its own (`0x00B0_xxxx`), below the primitive
/// (`0x00C0_xxxx`) and widget (`0x00D0_xxxx`) ranges and clear of the top-bit
/// namespace [`user_component_id`] hashes into.
pub const SLOT_OUTLET: ComponentId = ComponentId::from_raw(0x00B0_0001);

/// The tag that spells a §2.12 slot outlet: `<slot name="header"/>`.
///
/// A `&str` constant rather than a bare literal because three places must agree on
/// it and cannot be allowed to drift: [`builtin_component_id`] binds the tag,
/// [`attr_symbol_on`] rebinds `name` on it, and the frontend's `check_outlet`
/// rule decides what is legal to write on it.
///
/// Lowercase only, and deliberately: an outlet is a *position marker*, not a
/// component, and both front ends classify a lowercase tag as an element (see
/// [`TagPosition`]). A `<Slot/>` in component position stays what any unrecognised
/// component tag is — [`user_component_id("Slot")`](user_component_id), mounting
/// nothing — because giving the outlet two spellings would mean the front ends'
/// element/component classification no longer decided anything about it.
pub const OUTLET_TAG: &str = "slot";

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

/// Which authoring position a tag was written in — the component-vs-element
/// distinction both front ends already make.
///
/// This is an argument rather than something [`component_id_for`] infers,
/// because the two positions have genuinely different fallbacks and the resolver
/// used to pick one of them for everything. It is not re-derived from the tag's
/// spelling here: the front ends own that rule (`.okui` lowering's
/// `starts_uppercase`, the macro's `is_component`), they record their verdict on
/// the IR node's [`UiIrNodeKind`](crate::document::UiIrNodeKind), and a second
/// copy of the rule in this crate could only ever drift from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TagPosition {
    /// An element tag — `div`, `span`, `header`, `img`. Falls back to the
    /// generic [`CONTAINER`] box.
    Element,
    /// A component tag — `Button`, `Card`. Falls back to [`user_component_id`].
    Component,
}

/// The [`ComponentId`] of the built-in this tag names, or [`None`] when it names
/// no built-in.
///
/// Built-in widgets and non-container primitives resolve to their stable
/// `COMPONENT_ID`; the lowercase `img`/`canvas`/… element spellings resolve to
/// the same primitive as their PascalCase component spelling.
///
/// [`OUTLET_TAG`] is the one entry here that is not a component at all: nothing is
/// ever registered under [`SLOT_OUTLET`], and no node ever retains it — projection
/// substitutes it away inside a component's render, and the reconciler refuses it
/// anywhere else. It is a built-in because it is a *reserved tag*: `<slot>` must
/// mean the outlet rather than a `div` named `slot`, and this function is where
/// every reserved tag is stated.
#[must_use]
pub fn builtin_component_id(tag: &str) -> Option<ComponentId> {
    let id = match tag {
        // The §2.12 slot outlet — a position, not a component (see above).
        OUTLET_TAG => SLOT_OUTLET,
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
        // Every other tag names no built-in; what it binds to depends on the
        // position it was written in (see `component_id_for`).
        _ => return None,
    };
    Some(id)
}

/// Resolves a tag name, written in `position`, to the [`ComponentId`] it binds
/// to.
///
/// A built-in's **id** always wins: `<Button/>` resolves to the shipped widget's
/// `COMPONENT_ID` rather than to a hash of the string, and `img`/`Image` are the
/// same primitive. That is a claim about the handle, not about what renders — a
/// host that registers its own factory under `Button::COMPONENT_ID` (which is
/// what `#[contract(name = "Button")]` derives) takes the tag over, and this
/// function neither knows nor prevents it.
///
/// Otherwise the fallback follows the position — an element tag is the generic
/// [`CONTAINER`] box, a component tag is its own [`user_component_id`].
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

/// The [`ComponentId`] a **user** component tag binds to: FNV-1a 64 of the name,
/// lifted into a high namespace (top bit set).
///
/// This is the id `#[derive(Contract)]` puts on `ComponentContract::id` for the
/// same name, and the whole reason an author's `<Card/>` and a host's
/// `register_component(…)` meet without either consulting the other: the name is
/// the shared secret. The derive does not re-implement it — it calls
/// [`component_id_for`] — so the two cannot drift.
///
/// The top bit is set for the same reason [`fallback_symbol`] sets it: every
/// built-in `COMPONENT_ID` is a small low-range constant, so a hashed name can
/// never *alias* one. That makes the two namespaces disjoint by construction
/// rather than by the odds on a 64-bit hash.
#[must_use]
pub fn user_component_id(tag: &str) -> ComponentId {
    ComponentId::from_raw(hash_name_64(tag) | 0x8000_0000_0000_0000)
}

/// Resolves an attribute name written **on `tag`** to the [`InternedSymbol`] it
/// binds to.
///
/// This is [`attr_symbol`] plus the one binding that depends on the tag the
/// attribute sits on: on [`OUTLET_TAG`], `name` is the *slot the outlet stands
/// for*, so it binds to the reserved [`SLOT`] symbol that `project_slots` reads —
/// exactly as a child's `slot="…"` does, because the two are the two halves of one
/// join and must be the same symbol or they never meet. On every other tag `name`
/// is an ordinary hashed prop, and `<slot class="x">` is an ordinary hashed prop
/// too: only the pair (`slot`, `name`) is special.
///
/// # The obstacle that was not one
///
/// The audit recorded that an outlet "cannot cheaply become" a tag because
/// "`attr_symbol` binds an attribute by its own spelling with no idea which tag it
/// sits on, so `name` cannot mean the slot's name on one tag and a hashed prop
/// elsewhere". The first clause is true of the *function*; the conclusion does not
/// follow. Of [`attr_symbol`]'s three production callers, the two that lower a
/// **document** — `UiIrViewBridge::lower_element` and `ViewCodegen::emit_element` —
/// hold `node.tag()` in the same scope in which they call it, and each is already
/// threading `TagPosition` out of that same node. The tag was not absent; it was
/// one argument away. That is a much cheaper finding than the one the audit wrote,
/// and the difference between the two is the whole cost of this feature.
///
/// The third caller is the one the audit's claim is true of, and it is why
/// [`attr_symbol`] is still here rather than replaced: `#[derive(Contract)]` hashes
/// a prop name out of a struct field, where there is no tag by construction. It
/// keeps the tagless function, which is exactly this one's non-outlet branch, so
/// the two cannot disagree.
#[must_use]
pub fn attr_symbol_on(tag: &str, name: &str) -> InternedSymbol {
    if tag == OUTLET_TAG && name == "name" {
        return SLOT;
    }
    attr_symbol(name)
}

/// Resolves an attribute name to the [`InternedSymbol`] it binds to, with no tag
/// context.
///
/// The reserved CSS/state attributes map to the `obscur-view` symbols the
/// reconciler and matcher read; any other name is hashed into the high symbol
/// namespace (see [`fallback_symbol`]).
///
/// Prefer [`attr_symbol_on`] when the tag is known: one binding (`name` on a
/// `<slot>`) depends on it, and this function cannot see it.
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
        // A §2.12 slot assignment — structure the runtime routes on, not a
        // selector input.
        "slot" => SLOT,
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

/// Deterministic FNV-1a 64-bit hash of a name — the [`ComponentId`]-width
/// counterpart of [`hash_name_32`].
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
    fn reserved_attributes_map_to_reserved_symbols() {
        assert_eq!(attr_symbol("class"), CLASS);
        assert_eq!(attr_symbol("id"), ID);
        assert_eq!(attr_symbol("type"), TYPE);
        assert_eq!(attr_symbol("src"), SRC);
        assert_eq!(attr_symbol("disabled"), DISABLED);
        assert_eq!(attr_symbol("checked"), CHECKED);
        assert_eq!(attr_symbol("test-id"), TEST_ID);
        assert_eq!(attr_symbol("automation-id"), AUTOMATION_ID);
        assert_eq!(attr_symbol("slot"), SLOT);
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
            TEST_ID,
            AUTOMATION_ID,
            SLOT,
        ] {
            assert_eq!(symbol.raw() & 0x8000_0000, 0);
        }
    }
}
