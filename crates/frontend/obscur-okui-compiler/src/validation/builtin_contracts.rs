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
        //
        // The authored spelling of the selection is `checked`, not `selected`:
        // `selected` is the name of the *builder's* method, while `checked` is the
        // name of the thing on the node — and only `checked` is also what the
        // cascade reads for `:checked`. This list said `selected` while no authored
        // `<radio>` rendered anything at all, so the two could not yet disagree;
        // `RadioProps` mounts one now, and a spelling that painted a filled dot
        // while `radio:checked` failed to match it would be the disagreement.
        ComponentContract::new("Radio")
            .requiring_prop("label")
            .allowing_prop("checked")
            .allowing_prop("disabled")
            .allowing_event("change")
            .no_children(),
        // `Select::new(command)` + `.with_options(..)`/`.select(..)`/`.open()`/
        // `.disabled(..)`; one root click covers the whole control (header and rows
        // alike). `options` is a comma-separated list of labels, because
        // `with_options` takes labels rather than views — see `SelectProps`.
        ComponentContract::new("Select")
            .allowing_prop("options")
            .allowing_prop("selected")
            .allowing_prop("open")
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
        // `Progress::new(value)` — a fraction in `0..=1`, and the ONE attribute
        // this bar takes. `indeterminate` is not a prop but the absence of one:
        // `<progress/>` with no value is `Progress::indeterminate()`, the way
        // HTML reads the same markup. `shaped(Ring)` and `along(path)` are left
        // out deliberately — a shape is an enum with no authored spelling and a
        // path is SVG data, so admitting either would be a vocabulary the
        // lowering cannot yet turn into the `Int` / path the painter reads.
        //
        // No event surface: a bar reports and accepts nothing, and stating an
        // empty `allowed_events` would leave events unrestricted rather than
        // forbidden (see `check_props`) — which is the same open surface every
        // other non-interactive contract here has.
        ComponentContract::new("Progress")
            .allowing_prop("value")
            .no_children(),
        // `Slider::new(value)` + `.over(min, max)` + `.stepped_by(step)`, one
        // attribute each; binds `input`, which is what the runtime's drag and its
        // arrow keys dispatch (`report_slider`).
        //
        // `Slider::ranging(low, high)` is deliberately NOT authorable. A two-thumb
        // slider needs a third fact — `SLIDER_RANGED` — that decides which thumb a
        // press grabs and whether the ends may cross, and no authored attribute
        // states it; a `low`/`high` pair without it would be a control whose two
        // thumbs move together. Because this surface is CLOSED, `<Slider low=…>`
        // is a compile error that names the attribute rather than a slider that
        // quietly has one thumb.
        ComponentContract::new("Slider")
            .allowing_prop("value")
            .allowing_prop("min")
            .allowing_prop("max")
            .allowing_prop("step")
            .allowing_event("input")
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

#[doc = include_str!("builtin_contracts.doc/mounted_element_contracts.md")]
#[must_use]
pub fn mounted_element_contracts() -> Vec<ComponentContract> {
    ["input", "textarea", "checkbox", "radio", "select"]
        .into_iter()
        .map(|tag| ComponentContract::new(tag).no_children())
        .collect()
}

#[doc = include_str!("builtin_contracts.doc/default_contracts.md")]
#[must_use]
pub fn default_contracts() -> Vec<ComponentContract> {
    builtin_contracts()
        .into_iter()
        .chain(widget_contracts())
        .chain(mounted_element_contracts())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        ComponentContract, builtin_contracts, default_contracts, mounted_element_contracts,
        widget_contracts,
    };

    /// The contract for `tag`, from the widget set.
    fn widget(tag: &str) -> ComponentContract {
        widget_contracts()
            .into_iter()
            .find(|contract| contract.component() == tag)
            .unwrap_or_else(|| panic!("a {tag} contract"))
    }

    #[test]
    fn covers_every_shipped_widget() {
        // All fourteen tags `name_binding` resolves to a widget COMPONENT_ID.
        //
        // 2026-08-07. This asserted twelve, and the two it was missing are the two
        // that had no authored spelling at all: `<Progress/>` and `<Slider/>` bound
        // to a hashed id nobody registered, so there was nothing for a contract to
        // check. They are tags now, so they are contracts now.
        let contracts = widget_contracts();
        let names: Vec<&str> = contracts.iter().map(ComponentContract::component).collect();
        for expected in [
            "Button", "Input", "Checkbox", "Radio", "Select", "Dialog", "List", "Menu", "Tabs",
            "Tree", "Table", "Docking", "Progress", "Slider",
        ] {
            assert!(
                names.contains(&expected),
                "missing widget contract for {expected}"
            );
        }
        assert_eq!(
            names.len(),
            14,
            "exactly the shipped widgets, no invented tags"
        );
    }

    #[test]
    fn the_two_controls_mirror_their_builders_and_close_their_surfaces() {
        // `Progress::new(value)` and `Slider::new(value).over(min, max)
        // .stepped_by(step)`: one attribute per builder argument, and nothing
        // else. Closing the surface is the point — an authored `<Slider valu=…>`
        // has to be a named compile error, because the alternative is the defect
        // this whole stage exists to end: an attribute that lowers, travels the
        // tree, and is read by nobody.
        let progress = widget("Progress");
        assert!(progress.required_props().is_empty(), "a bare bar is legal");
        assert_eq!(progress.allowed_props(), &["value".into()]);
        assert!(
            progress.allowed_events().is_empty(),
            "a bar reports; it accepts nothing"
        );

        let slider = widget("Slider");
        assert!(slider.required_props().is_empty(), "0..1 is a real default");
        for prop in ["value", "min", "max", "step"] {
            assert!(slider.allowed_props().contains(&prop.into()), "{prop}");
        }
        // The state the markup deliberately cannot reach. `Slider::ranging` needs
        // `SLIDER_RANGED` as well as two ends, and no attribute states it — so the
        // closed surface turns the attempt into an error rather than into a slider
        // whose two thumbs silently move together.
        for absent in ["low", "high", "ranged"] {
            assert!(!slider.allowed_props().contains(&absent.into()), "{absent}");
        }
        // `input` and only `input`: it is what `report_slider` dispatches on a
        // drag and on an arrow key alike.
        assert_eq!(slider.allowed_events(), &["input".into()]);
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
    fn radio_spells_its_selection_the_way_the_node_does() {
        // `selected` names the builder's method; `checked` names the thing on the
        // node, and it is the one the cascade reads for `:checked`. Now that an
        // authored `<Radio/>` mounts a control that paints from that state, the two
        // spellings could disagree — so there is only one.
        let radio = widget("Radio");
        assert_eq!(radio.required_props(), &["label".into()]);
        assert!(radio.allowed_props().contains(&"checked".into()));
        assert!(
            !radio.allowed_props().contains(&"selected".into()),
            "a second spelling of the selection would be a second answer to \
             `radio:checked`"
        );
    }

    #[test]
    fn select_states_its_options_as_a_prop_rather_than_as_children() {
        // The options are labels, not views (`Select::with_options` takes
        // `Into<Box<str>>`), and the control synthesises each `.select-option` row
        // around them. So they are a prop, and the child surface stays closed —
        // an authored `<option>` would be a vocabulary nothing renders.
        let select = widget("Select");
        assert!(select.required_props().is_empty(), "a bare select is legal");
        for prop in ["options", "selected", "open", "disabled"] {
            assert!(
                select.allowed_props().contains(&prop.into()),
                "Select must admit {prop}"
            );
        }
        assert!(select.children().is_none());
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
    fn a_base_element_whose_factory_owns_its_children_forbids_authored_ones() {
        // `BuiltinComponents` registers a factory whose component owns its
        // children for `<input>`, `<textarea>`, `<checkbox>`, `<radio>` and
        // `<select>`, and a mounted component's output REPLACES its host's
        // children — so an authored child under one of these tags is thrown away
        // by the frame that renders it. Silently, and because of a registration
        // made somewhere else. These five entries turn that into a compile error
        // that names the tag. `<button>` mounts too and is deliberately NOT here:
        // its component renders the default-slot outlet, so the author's children
        // are projected back rather than replaced.
        let contracts = mounted_element_contracts();
        let names: Vec<&str> = contracts.iter().map(ComponentContract::component).collect();
        assert_eq!(
            names,
            vec!["input", "textarea", "checkbox", "radio", "select"]
        );
        for contract in &contracts {
            assert!(
                contract.children().is_none(),
                "{} must forbid children",
                contract.component()
            );
            // Only the child policy. Declaring a prop would CLOSE the prop surface
            // (see `check_props`), rejecting attributes these tags accept today.
            assert!(
                contract.required_props().is_empty() && contract.allowed_props().is_empty(),
                "{} states a prop surface it was not asked for",
                contract.component()
            );
        }
    }

    #[test]
    fn the_default_table_is_the_three_tables_and_nothing_else() {
        // What a default compile knows about, stated once. The order is the claim
        // that matters: a registry keyed by tag string lets a later entry win, and
        // the base-element policies are last so that a future collision resolves
        // towards the tag whose factory owns its children.
        let default = default_contracts();
        assert_eq!(
            default.len(),
            builtin_contracts().len()
                + widget_contracts().len()
                + mounted_element_contracts().len()
        );
        let names: Vec<&str> = default.iter().map(ComponentContract::component).collect();
        assert_eq!(names.first(), Some(&"Container"));
        assert_eq!(names.last(), Some(&"select"));
    }

    #[test]
    fn the_tags_that_mount_nothing_keep_their_children() {
        // The boundary, stated from the other side, and for two DIFFERENT reasons:
        // `<table>` renders its authored children because no factory claims that id
        // at all, while `<button>` renders them although a factory does — its
        // component is the §2.12 default-slot outlet, so the children are projected
        // back rather than replaced. A contract here would forbid markup that works
        // in both cases, so the rule is "whose factory owns its children", not
        // "every base element" and not "every unmounted one".
        let contracts = mounted_element_contracts();
        let names: Vec<&str> = contracts.iter().map(ComponentContract::component).collect();
        for tag in ["button", "table", "div", "img"] {
            assert!(!names.contains(&tag), "{tag} must stay permissive");
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
