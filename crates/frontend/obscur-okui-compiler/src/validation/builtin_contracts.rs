//! The default, authoritative contract set for the built-in view primitives
//! ([`builtin_contracts`]) and the shipped widgets ([`widget_contracts`])
//! (spec §2.6, §2.13).
//!
//! Spec §2.13 requires that unknown attributes and events on a *known*
//! component are compile errors. For that check to bite on a real compile, the
//! [`ContractValidator`](super::ContractValidator) must consult a **populated**
//! registry rather than an empty one; this module supplies both sets so the
//! default compile path enforces their attribute/event surface out of the box.
//!
//! # Why the surface is mirrored here rather than imported
//!
//! The primitives live in `obscur-primitives` as plain view-builder config
//! structs and deliberately **do not** export a component contract (they carry
//! only stable [`ComponentId`]/prop-symbol constants). The typed
//! `obscur_view::component::ComponentContract` names every prop/event by an
//! opaque, hash-derived `InternedSymbol` with no reverse string resolution, so
//! it cannot drive the `.okui` string-keyed lookup either. The authoritative
//! *string* surface for these primitives is therefore their spec-defined
//! builder API (spec §2.6): each contract below mirrors one primitive's
//! attributes/events exactly.
//!
//! [`ComponentId`]: obscur_identity::ComponentId

use crate::validation::ComponentContract;

/// Returns the contracts for every built-in view primitive (spec §2.6), keyed
/// later by their tag name in the [`ContractRegistry`](crate::compile::ContractRegistry).
///
/// The set mirrors the `obscur-primitives` builder surfaces:
///
/// - `Container` — a grouping box: no attributes of its own, any children.
/// - `Text` — a literal text run: no attribute surface (its content is child
///   text), any children.
/// - `Image` — requires `src`, optionally `alt`; a leaf.
/// - `Scroll` — optional `horizontal`/`vertical` axis flags, any children.
/// - `Canvas` — requires `width` and `height`; a leaf.
/// - `Portal` — requires `target`, any children.
/// - `Layer` — optional `opacity`, any children.
/// - `ForeignSurface` — requires `surface`; a leaf.
///
/// A primitive that declares no attributes (`Container`, `Text`) leaves its
/// prop surface *open* — the string contract model treats "no declared props"
/// as unrestricted — so those tags stay permissive; every primitive that
/// declares props closes its surface and rejects unknown attributes.
#[must_use]
pub fn builtin_contracts() -> Vec<ComponentContract> {
    vec![
        ComponentContract::new("Container"),
        ComponentContract::new("Text"),
        ComponentContract::new("Image")
            .requiring_prop("src")
            .allowing_prop("alt"),
        ComponentContract::new("Scroll")
            .allowing_prop("horizontal")
            .allowing_prop("vertical"),
        ComponentContract::new("Canvas")
            .requiring_prop("width")
            .requiring_prop("height"),
        ComponentContract::new("Portal").requiring_prop("target"),
        ComponentContract::new("Layer").allowing_prop("opacity"),
        ComponentContract::new("ForeignSurface").requiring_prop("surface"),
    ]
}

/// Returns the contracts for every shipped widget (spec §2.6, §2.13), keyed
/// later by tag name exactly like [`builtin_contracts`].
///
/// All twelve widget tags already bind to a real `ComponentId` through
/// `obscur_ui_ir::name_binding`, but none carried a contract — so
/// `<Button laebl="OK"/>` compiled clean. These close that hole for the widgets
/// that have an authored surface.
///
/// # Where the surface comes from
///
/// Same rule, and same reason, as the primitives above: `obscur-okui-compiler`
/// is a *frontend* crate and cannot depend on the components tier, so the
/// surfaces are mirrored rather than imported. The widgets' own prop symbols
/// could not drive this even if they were reachable — they are private, and they
/// name the state a widget *writes out* (`role`, `pressed`, `selected`), not the
/// attributes an author *writes in*. The authoritative string surface is
/// therefore each widget's §2.6 builder API: a positional `new()` argument with
/// no default is a **required** prop, and each fluent setter is an **allowed**
/// one.
///
/// # Deliberate omissions
///
/// - **`command` is not a prop.** Every interactive widget takes
///   `command: InternedSymbol` in its constructor, but the view binds it as the
///   event *handler*, so it is authored as `on:click=submit`, never as an
///   attribute.
/// - **Collection widgets declare no props.** `Menu`, `Tabs`, `Tree`, `Table`
///   and `Docking` take their content as typed Rust values (`MenuItem`,
///   `TreeNode`, `Column`, `DockPanel`, labels) with no string spelling at all.
///   Inventing an `<Option>` / `<Tab>` / `<Column>` child vocabulary would be new
///   design, not mirroring, so they are registered with an **open** surface (as
///   `Container`/`Text` are) and stay unchecked until that vocabulary is
///   actually specified.
/// - **"No children" is not expressible.** Only `Dialog` accepts authored
///   children; every other widget synthesises its children inside `view()` from
///   its own model, so `<Button>OK</Button>` is meaningless. An empty
///   `allowed_children` means *any child*, and the contract type has no way to
///   say *none*, so that misuse still passes. Closing it needs a type change.
#[must_use]
pub fn widget_contracts() -> Vec<ComponentContract> {
    vec![
        // `Button::new(label, command)` + `.disabled(..)`; binds EVENT_CLICK.
        ComponentContract::new("Button")
            .requiring_prop("label")
            .allowing_prop("disabled")
            .allowing_event("click"),
        // `Input::new(command)` — nothing positional but the command — plus
        // `.with_value/.placeholder/.disabled/.readonly/.multiline`. Binds both
        // `input` and `change` to the same command.
        ComponentContract::new("Input")
            .allowing_prop("value")
            .allowing_prop("placeholder")
            .allowing_prop("disabled")
            .allowing_prop("readonly")
            .allowing_prop("multiline")
            .allowing_event("input")
            .allowing_event("change"),
        // `Checkbox::new(label, command)` + `.checked(..)`/`.disabled(..)`.
        ComponentContract::new("Checkbox")
            .requiring_prop("label")
            .allowing_prop("checked")
            .allowing_prop("disabled")
            .allowing_event("change"),
        // `Radio::new(label, command)` + `.selected(..)`/`.disabled(..)`.
        ComponentContract::new("Radio")
            .requiring_prop("label")
            .allowing_prop("selected")
            .allowing_prop("disabled")
            .allowing_event("change"),
        // `Select::new(command)` + `.disabled(..)`; one root click covers the
        // whole control (header and rows alike).
        ComponentContract::new("Select")
            .allowing_prop("disabled")
            .allowing_event("click"),
        // `Dialog::new(title)` + `.modal(..)`. The ONLY widget whose children are
        // authored rather than synthesised — and they are optional.
        ComponentContract::new("Dialog")
            .requiring_prop("title")
            .allowing_prop("modal"),
        // `List::new()` + `.multi(..)`; selection is driven by Rust calls, not by
        // an event binding.
        ComponentContract::new("List").allowing_prop("multi"),
        // Open surface — see "Deliberate omissions" above.
        ComponentContract::new("Menu"),
        ComponentContract::new("Tabs"),
        ComponentContract::new("Tree"),
        ComponentContract::new("Table"),
        ComponentContract::new("Docking"),
    ]
}

#[cfg(test)]
mod tests {
    use super::{ComponentContract, builtin_contracts, widget_contracts};

    /// The contract for `tag`, from the widget set.
    fn widget(tag: &str) -> ComponentContract {
        widget_contracts()
            .into_iter()
            .find(|contract| contract.component() == tag)
            .unwrap_or_else(|| panic!("a {tag} contract"))
    }

    #[test]
    fn covers_every_shipped_widget() {
        // All twelve tags `name_binding` resolves to a widget COMPONENT_ID.
        let contracts = widget_contracts();
        let names: Vec<&str> = contracts.iter().map(ComponentContract::component).collect();
        for expected in [
            "Button", "Input", "Checkbox", "Radio", "Select", "Dialog", "List", "Menu", "Tabs",
            "Tree", "Table", "Docking",
        ] {
            assert!(names.contains(&expected), "missing widget contract for {expected}");
        }
        assert_eq!(names.len(), 12, "exactly the shipped widgets, no invented tags");
    }

    #[test]
    fn button_mirrors_its_builder_api() {
        // `Button::new(label, command)` + `.disabled(..)`: label is positional
        // with no default (required), disabled is a fluent setter (allowed).
        let button = widget("Button");
        assert_eq!(button.required_props(), &["label".into()]);
        assert_eq!(button.allowed_props(), &["disabled".into()]);
        assert_eq!(button.allowed_events(), &["click".into()]);
    }

    #[test]
    fn command_is_never_mirrored_as_a_prop() {
        // Every interactive widget takes `command` in its constructor, but it is
        // bound as the event HANDLER (`on:click=submit`), not an attribute — so
        // it must not appear on any prop surface.
        for contract in widget_contracts() {
            let props: Vec<_> = contract
                .required_props()
                .iter()
                .chain(contract.allowed_props())
                .collect();
            assert!(
                !props.iter().any(|prop| &***prop == "command"),
                "{} mirrors `command` as a prop",
                contract.component()
            );
        }
    }

    #[test]
    fn input_closes_its_prop_surface_without_requiring_anything() {
        // `Input::new(command)` has no positional prop, so nothing is required —
        // but declaring any prop still CLOSES the surface, which is the point: a
        // typo'd attribute is rejected even though no attribute is mandatory.
        let input = widget("Input");
        assert!(input.required_props().is_empty());
        assert!(input.allowed_props().contains(&"placeholder".into()));
        assert!(input.allowed_props().contains(&"multiline".into()));
        assert_eq!(input.allowed_events(), &["input".into(), "change".into()]);
    }

    #[test]
    fn collection_widgets_stay_open_rather_than_inventing_a_vocabulary() {
        // Menu/Tabs/Tree/Table/Docking take typed Rust content with no string
        // spelling; registering an open surface keeps them honestly unchecked
        // instead of asserting a vocabulary that does not exist.
        for tag in ["Menu", "Tabs", "Tree", "Table", "Docking"] {
            let contract = widget(tag);
            assert!(contract.required_props().is_empty(), "{tag} requires a prop");
            assert!(contract.allowed_props().is_empty(), "{tag} declares a prop");
        }
    }

    #[test]
    fn widget_and_primitive_contracts_do_not_collide() {
        // Both sets are registered into one tag-keyed registry, so an overlap
        // would silently shadow one of them.
        let primitives = builtin_contracts();
        let names: Vec<&str> = primitives.iter().map(ComponentContract::component).collect();
        for contract in widget_contracts() {
            assert!(
                !names.contains(&contract.component()),
                "{} is registered as both a primitive and a widget",
                contract.component()
            );
        }
    }

    #[test]
    fn covers_every_built_in_primitive() {
        let contracts = builtin_contracts();
        let names: Vec<&str> = contracts.iter().map(ComponentContract::component).collect();
        for expected in [
            "Container",
            "Text",
            "Image",
            "Scroll",
            "Canvas",
            "Portal",
            "Layer",
            "ForeignSurface",
        ] {
            assert!(
                names.contains(&expected),
                "missing built-in contract for {expected}"
            );
        }
    }

    #[test]
    fn image_requires_src_and_allows_alt() {
        let contracts = builtin_contracts();
        let image = contracts
            .iter()
            .find(|contract| contract.component() == "Image")
            .expect("an Image contract");
        assert_eq!(image.required_props(), &["src".into()]);
        assert_eq!(image.allowed_props(), &["alt".into()]);
    }

    #[test]
    fn container_leaves_its_prop_surface_open() {
        let contracts = builtin_contracts();
        let container = contracts
            .iter()
            .find(|contract| contract.component() == "Container")
            .expect("a Container contract");
        assert!(container.required_props().is_empty());
        assert!(container.allowed_props().is_empty());
    }
}
