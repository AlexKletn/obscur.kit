#![doc = include_str!("facade_boundary.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::headless::input::{SelectorResolver, SelectorTarget};
use obscur_kit::identity::InternedSymbol;

use facade_consumer::badge::{BadgeProps, badge_screen, badge_text, register_badge};
use facade_consumer::counter::{CMD_INCREMENT, COUNT_TEST_ID, PRESS_AUTOMATION_ID, counter_view};
use facade_consumer::measure::{find, measure, measure_in, runtime};

const CSS: &str = ".app { width: 100%; height: 100% } .count { height: 20px }";

#[test]
fn the_okui_macro_expands_in_a_crate_that_has_only_the_facade() {
    // Reaching the view proves the expansion produced a real one rather than an
    // empty fragment.
    let view = counter_view(7);
    assert_eq!(view.root().len(), 1);
}

#[test]
fn a_view_the_macro_built_lays_out_through_the_facade() {
    let nodes = measure(&counter_view(1), CSS, 480.0, 240.0);
    let app = find(&nodes, "app").expect("the authored root is in the snapshot");
    assert_eq!((app.width, app.height), (480.0, 240.0));
}

#[test]
fn a_hyphenated_attribute_the_macro_emitted_names_a_node_an_application_can_find() {
    // Audit finding 2-3, at the boundary it matters on. `test-id` / `automation-id`
    // are the §7.5 identifiers `App::on_command_at` and `App::add_event_listener`
    // target a node with — an identifier that exists *before* the tree does, which
    // is the whole reason those calls can be made while an application configures
    // itself. `okui!` could not express either, so an application on the primary
    // authoring surface had no way to write the name it then has to say.
    //
    // Compiling `src/counter.rs` is half the claim. The other half is that the
    // joined name reached the *reserved* symbol: `SelectorResolver` looks under
    // `TEST_ID`/`AUTOMATION_ID` and under nothing else, so a name that hashed
    // instead would resolve to `None` here and to a handler that never runs in an
    // application.
    let mut runtime = runtime(480.0, 240.0);
    let nodes = measure_in(&mut runtime, &counter_view(1), CSS);
    assert!(!nodes.is_empty(), "the tree was laid out");

    let resolver = SelectorResolver::new();
    let readout = resolver.resolve(&runtime, &SelectorTarget::test_id(COUNT_TEST_ID));
    let button = resolver.resolve(
        &runtime,
        &SelectorTarget::automation_id(PRESS_AUTOMATION_ID),
    );
    assert!(
        readout.is_some(),
        "`test-id` in `okui!` markup resolved against the live tree"
    );
    assert!(
        button.is_some(),
        "and so did `automation-id`, which binds to a different reserved symbol"
    );
    assert_ne!(readout, button, "two names, two nodes");
    // A name nobody authored still answers `None`, so the two above are matches
    // and not a resolver that says yes to everything.
    assert_eq!(
        resolver.resolve(&runtime, &SelectorTarget::test_id("no-such-row")),
        None
    );
}

// --- The component tier: `#[derive(Contract)]` (audit finding 2-2). --------

#[test]
fn a_component_an_application_declared_mounts_and_renders() {
    // Compiling `src/badge.rs` is most of this claim — the derive used to emit
    // `::obscur_view::…`, which is `error[E0433]` in any crate but this
    // workspace's own. The rest is the claim that the declaration is *live*: the
    // id the host registers under (from `contract()`) and the id `<Badge/>`
    // lowers to are computed by two different macros from one name string, and
    // nothing but their agreement makes the component mount.
    let mut runtime = runtime(240.0, 120.0);
    register_badge(&mut runtime);
    let nodes = measure_in(&mut runtime, &badge_screen("PING", 3), "");

    assert_eq!(
        runtime.mounted_component_count(),
        1,
        "only <Badge/> names the registered component — a <div>/<span> is a box"
    );
    // Its output is a subtree this crate never authored, so its text is the only
    // evidence it ran at all.
    let rendered: Vec<&str> = nodes.iter().map(|node| node.text.as_str()).collect();
    assert!(
        rendered.contains(&badge_text("PING").as_str()),
        "the component rendered and decoded its required prop: {rendered:?}"
    );
    assert!(
        rendered.contains(&"3"),
        "the `#[contract(default)]` prop decoded the authored value: {rendered:?}"
    );
    // Authored children reached the slot the props struct declares.
    assert!(
        rendered.contains(&"details"),
        "slot content projected into the component's outlet: {rendered:?}"
    );
    // And the sibling box is still a box: it rendered its own child and nothing
    // of the component's.
    assert!(find(&nodes, "plain").is_some());
}

#[test]
fn an_applications_stylesheet_reaches_the_subtree_its_component_rendered() {
    // Audit finding 2-1, from the vantage point it was found at. The application
    // authors `.badge { … }` in an ordinary sheet; `div.badge` is a node it never
    // wrote, because `Badge` rendered it. Before `StyleScope` existed there was
    // no way for that rule to reach it — not this rule, not a type selector, not
    // the UA sheet — and a component that cannot be styled cannot be a UI
    // component.
    //
    // The sibling `.plain` box carries the same declaration and is authored
    // inline, so the assertion is a comparison and not a number: two nodes,
    // identical apart from which of them a component produced, must measure the
    // same. A build where the sheet reaches neither would fail on the sibling
    // first.
    let mut runtime = runtime(240.0, 120.0);
    register_badge(&mut runtime);
    let nodes = measure_in(
        &mut runtime,
        &badge_screen("PING", 3),
        ".badge, .plain { width: 64px; height: 24px }",
    );

    let plain = find(&nodes, "plain").expect("the authored sibling is in the snapshot");
    assert_eq!(
        (plain.width, plain.height),
        (64.0, 24.0),
        "the control: an authored node takes the rule"
    );
    let badge = find(&nodes, "badge").expect("the component's own node is in the snapshot");
    assert_eq!(
        (badge.width, badge.height),
        (plain.width, plain.height),
        "a node the component rendered must take the same rule — it declares \
         `#[contract(style_scope = open)]`"
    );
}

#[test]
fn a_derived_contract_is_the_declaration_the_author_wrote() {
    // The string surface the front ends check a document against, reachable from
    // an application because the derive emits it through the facade too.
    let surface = BadgeProps::contract_surface();
    assert_eq!(surface.component(), "Badge");
    assert_eq!(surface.required_props(), ["label"]);
    assert_eq!(surface.allowed_props(), ["hint", "count"]);
    assert_eq!(surface.allowed_events(), ["pressed"]);
    assert_eq!(BadgeProps::contract().id, authored_tag_id());
}

#[doc = include_str!("facade_boundary.doc/authored_tag_id.md")]
fn authored_tag_id() -> obscur_kit::identity::ComponentId {
    obscur_kit::ui_ir::name_binding::component_id_for(
        "Badge",
        obscur_kit::ui_ir::name_binding::TagPosition::Component,
    )
}

#[test]
fn an_application_can_name_a_command_the_markup_binds_to() {
    // The name in `on:click={increment}` and the symbol an application registers a
    // handler under have to be the same handle, and an application can only reach
    // the latter through the facade.
    assert_eq!(CMD_INCREMENT, InternedSymbol::named("increment"));
}

// --- Custom properties across the facade boundary. -------------------------

#[test]
fn a_system_theme_variable_reads_back_through_var_in_a_crate_that_has_only_the_facade() {
    // The defect this guards, in the shape it shipped in: `install_system_theme`
    // compiles a `:root { --obscur-sys-…: … }` block into the runtime, and an
    // application reads one with `var()`. That worked for as long as a
    // single-component declaration survived compilation as a typed value. Once a
    // custom property began keeping its raw tokens, reading one back needed the
    // substitution-time grammar — which exactly one call installed, and it was
    // not this one. Every desktop colour then computed to `unset`.
    //
    // Neither fixture had a `var(` in it, which is why the guard that exists
    // precisely to catch what only a real consumer sees did not catch this.
    let mut runtime = runtime(240.0, 120.0);
    let appearance = obscur_kit::system::appearance::SystemAppearance {
        cursor_size: Some(37),
        ..obscur_kit::system::appearance::SystemAppearance::default()
    };
    let vars =
        obscur_kit::bootstrap::KitBootstrap::new().install_system_theme(&mut runtime, &appearance);
    assert_eq!(
        vars.get("--obscur-sys-cursor-size"),
        Some("37px"),
        "the kit wrote the variable it is about to be asked to read"
    );

    let nodes = measure_in(
        &mut runtime,
        &counter_view(1),
        ".count { height: var(--obscur-sys-cursor-size) }",
    );
    let count = find(&nodes, "count").expect("the styled row is in the snapshot");
    assert_eq!(
        count.height, 37.0,
        "the value the kit itself wrote resolved; `unset` here is the regression"
    );
}

#[test]
fn a_variable_nobody_declared_does_not_measure_the_same_as_one_that_was() {
    // The control for the test above, and the reason its 37 is a substitution
    // rather than a coincidence: the same sheet naming a variable nobody
    // declared is invalid at computed-value time, which is `unset` — `auto` for
    // `height`, so the row takes its content height instead.
    let mut runtime = runtime(240.0, 120.0);
    obscur_kit::bootstrap::KitBootstrap::new().install_system_theme(
        &mut runtime,
        &obscur_kit::system::appearance::SystemAppearance {
            cursor_size: Some(37),
            ..obscur_kit::system::appearance::SystemAppearance::default()
        },
    );
    let nodes = measure_in(
        &mut runtime,
        &counter_view(1),
        ".count { height: var(--obscur-sys-no-such-variable) }",
    );
    let count = find(&nodes, "count").expect("the row is in the snapshot");
    assert_ne!(
        count.height, 37.0,
        "an undeclared name must not measure the same as a declared one"
    );
}

#[test]
fn an_applications_own_custom_property_resolves_after_the_bootstrap_installed_css() {
    // The commoner shape of the same wall, and why the funnel is drawn at "the
    // bootstrap installed CSS" rather than at the system theme alone: nothing
    // here is the kit's variable. An application installs the UA sheet, hands
    // the render path its own hand-compiled sheet, and never names
    // `CssPipeline` — which is what `measure_in` does, and what a host that
    // drives `UiRuntime` itself does.
    let mut runtime = runtime(240.0, 120.0);
    obscur_kit::bootstrap::KitBootstrap::new().install_default_stylesheet(&mut runtime);
    let nodes = measure_in(
        &mut runtime,
        &counter_view(1),
        ":root { --app-row-height: 41px } .count { height: var(--app-row-height) }",
    );
    let count = find(&nodes, "count").expect("the styled row is in the snapshot");
    assert_eq!(count.height, 41.0);
}
