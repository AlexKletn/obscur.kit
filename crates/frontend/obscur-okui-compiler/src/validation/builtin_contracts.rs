#![doc = include_str!("builtin_contracts.doc/_module.md")]

use crate::validation::ComponentContract;

#[doc = include_str!("builtin_contracts.doc/builtin_contracts.md")]
#[must_use]
pub fn builtin_contracts() -> Vec<ComponentContract> {
    vec![
        ComponentContract::new("Container"),
        ComponentContract::new("Text"),
        ComponentContract::new("Image")
            .requiring_prop("src")
            .allowing_prop("alt")
            .no_children(),
        ComponentContract::new("Scroll")
            .allowing_prop("horizontal")
            .allowing_prop("vertical"),
        ComponentContract::new("Canvas")
            .requiring_prop("width")
            .requiring_prop("height")
            .no_children(),
        ComponentContract::new("Portal").requiring_prop("target"),
        ComponentContract::new("Layer").allowing_prop("opacity"),
        ComponentContract::new("ForeignSurface")
            .requiring_prop("surface")
            .no_children(),
    ]
}

#[doc = include_str!("builtin_contracts.doc/widget_contracts.md")]
#[must_use]
pub fn widget_contracts() -> Vec<ComponentContract> {
    vec![
        // `Button::new(label, command)` + `.disabled(..)`; binds EVENT_CLICK.
        ComponentContract::new("Button")
            .requiring_prop("label")
            .allowing_prop("disabled")
            .allowing_event("click")
            .no_children(),
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
            .allowing_event("change")
            .no_children(),
        // `Checkbox::new(label, command)` + `.checked(..)`/`.disabled(..)`.
        ComponentContract::new("Checkbox")
            .requiring_prop("label")
            .allowing_prop("checked")
            .allowing_prop("disabled")
            .allowing_event("change")
            .no_children(),
        // `Radio::new(label, command)` + `.selected(..)`/`.disabled(..)`.
        ComponentContract::new("Radio")
            .requiring_prop("label")
            .allowing_prop("selected")
            .allowing_prop("disabled")
            .allowing_event("change")
            .no_children(),
        // `Select::new(command)` + `.disabled(..)`; one root click covers the
        // whole control (header and rows alike).
        ComponentContract::new("Select")
            .allowing_prop("disabled")
            .allowing_event("click")
            .no_children(),
        // `Dialog::new(title)` + `.modal(..)`. The ONLY widget whose children are
        // authored rather than synthesised — and they are optional, so its child
        // policy stays `Any`.
        ComponentContract::new("Dialog")
            .requiring_prop("title")
            .allowing_prop("modal"),
        // `List::new()` + `.multi(..)`; selection is driven by Rust calls, not by
        // an event binding.
        ComponentContract::new("List")
            .allowing_prop("multi")
            .no_children(),
        // Open PROP surface, leaf CHILD surface — see "No children" above: their
        // content is typed Rust values, so no authored child could name it.
        ComponentContract::new("Menu").no_children(),
        ComponentContract::new("Tabs").no_children(),
        ComponentContract::new("Tree").no_children(),
        ComponentContract::new("Table").no_children(),
        ComponentContract::new("Docking").no_children(),
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
            assert!(
                names.contains(&expected),
                "missing widget contract for {expected}"
            );
        }
        assert_eq!(
            names.len(),
            12,
            "exactly the shipped widgets, no invented tags"
        );
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
        // spelling; registering an open PROP surface keeps them honestly unchecked
        // instead of asserting a vocabulary that does not exist. Their child
        // surface is closed for the same reason — no authored child could name that
        // content, so any child is dead markup (see `every_widget_but_dialog_…`).
        for tag in ["Menu", "Tabs", "Tree", "Table", "Docking"] {
            let contract = widget(tag);
            assert!(
                contract.required_props().is_empty(),
                "{tag} requires a prop"
            );
            assert!(contract.allowed_props().is_empty(), "{tag} declares a prop");
        }
    }

    #[test]
    fn every_widget_but_dialog_takes_no_children() {
        // The §2.13 "allowed children" state that did not exist: eleven of the
        // twelve widgets build their children in their own `view()`, so an authored
        // child reaches no tree. `Dialog` is the one whose children ARE the content.
        for contract in widget_contracts() {
            let expected_leaf = contract.component() != "Dialog";
            assert_eq!(
                contract.children().is_none(),
                expected_leaf,
                "{}'s child policy is {:?}",
                contract.component(),
                contract.children()
            );
        }
    }

    #[test]
    fn the_leaf_primitives_declare_themselves_leaves() {
        // `Image`/`Canvas`/`ForeignSurface` were documented as leaves and admitted
        // any child; the other five primitives are genuine containers.
        for contract in builtin_contracts() {
            let expected_leaf =
                matches!(contract.component(), "Image" | "Canvas" | "ForeignSurface");
            assert_eq!(
                contract.children().is_none(),
                expected_leaf,
                "{}'s child policy is {:?}",
                contract.component(),
                contract.children()
            );
        }
    }

    #[test]
    fn widget_and_primitive_contracts_do_not_collide() {
        // Both sets are registered into one tag-keyed registry, so an overlap
        // would silently shadow one of them.
        let primitives = builtin_contracts();
        let names: Vec<&str> = primitives
            .iter()
            .map(ComponentContract::component)
            .collect();
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
