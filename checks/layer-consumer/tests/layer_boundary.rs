#![doc = include_str!("layer_boundary.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::headless::input::{SelectorResolver, SelectorTarget};
use obscur_kit::windowing::layer::{
    LayerBand, LayerEdge, LayerFocusPolicy, LayerReservation, LayerSettings,
};

use layer_consumer::key::{keyboard_view, register_key, sample_rows};
use layer_consumer::measure::{find, measure_in, runtime};
use layer_consumer::panel::{CSS, KEYBOARD_TEST_ID, docked, floating};
use layer_consumer::probe::{DisplayChoice, choose_display, first_display};

// What compiling this crate at all already proves, and what these tests add.
//
// The compile is the larger half. `panel::run_keyboard` composes the whole layer
// entry point — `App::layer`, `layer_control`, `control`, `on_command_at`,
// `add_event_listener`, `run` — in a feature set with NO `window-winit`, which is
// the only configuration where the layer runner is the sole reader of what
// `AppParts::boot` produces. It is deliberately never called: there is no
// compositor here, and running it is not what it is for.
//
// These tests cover the half a compile cannot: that the settings mean what the
// application thinks, that the component the markup names is the one it
// registered, and that the name `on_command_at` targets is a name that resolves.

#[test]
fn a_keyboard_docks_to_the_edge_it_asked_for_and_reserves_its_own_height() {
    let settings = docked();
    assert_eq!(settings.docked_edge(), Some(LayerEdge::Bottom));
    assert_eq!(settings.reservation, LayerReservation::Reserve(320));
    assert!(settings.reserves_space());
    // The stretched axis is the platform's to give, and stays unstated rather
    // than being sent as a zero.
    assert_eq!(settings.height, Some(320));
    assert_eq!(settings.width, None);
}

#[test]
fn the_default_focus_policy_is_the_one_a_keyboard_needs() {
    // A keyboard that took the keyboard would swallow the very shortcuts it
    // exists to send. This is the setting such an application cannot afford to
    // forget, and it is the default — so it cannot be forgotten.
    assert_eq!(docked().focus, LayerFocusPolicy::Never);
    assert_eq!(LayerFocusPolicy::default(), LayerFocusPolicy::Never);
}

#[test]
fn floating_changes_only_what_it_says_it_changes() {
    let (docked, floating) = (docked(), floating());
    assert_eq!(floating.reservation, LayerReservation::None);
    assert!(!floating.reserves_space());
    // …and nothing else moved: same band, same edge, same thickness.
    assert_eq!(floating.band, docked.band);
    assert_eq!(floating.band, LayerBand::Overlay);
    assert_eq!(floating.docked_edge(), docked.docked_edge());
    assert_eq!(floating.height, docked.height);
}

#[test]
fn floating_the_keyboard_moves_it_without_asking_for_a_frame() {
    // Audit 2-H, asserted from outside the facade because it is a promise to
    // applications rather than an internal detail: this fixture's own two
    // settings differ by a margin and a reservation, and a drag is a stream of
    // exactly that. Answering each with a frame re-renders identical content
    // once per position.
    assert!(!floating().needs_redraw_from(&docked()));
    assert!(!docked().needs_redraw_from(&floating()));
}

#[test]
fn making_the_keyboard_taller_does_ask_for_one() {
    // The other side, so the test above cannot pass by the method answering
    // `false` to everything.
    let taller = LayerSettings::bottom_bar(400);
    assert!(taller.needs_redraw_from(&docked()));
}

#[test]
fn a_component_this_crate_declared_mounts_inside_two_nested_loops() {
    // The exact markup shape `docs/layer-consumer-audit.md` withdrew a suspicion
    // about, kept as a fixture so the suspicion cannot come back unmeasured: a
    // `<For>` inside a `<For>`, whose body is a component tag taking expression
    // props. Five keys across two rows.
    let mut runtime = runtime(640.0, 320.0);
    register_key(&mut runtime);
    let rows = sample_rows();
    let nodes = measure_in(&mut runtime, &keyboard_view(&rows), CSS);

    let authored: usize = rows.iter().map(|row| row.keys.len()).sum();
    assert_eq!(authored, 5, "the fixture's own model, so the count below means something");
    assert_eq!(
        runtime.mounted_component_count(),
        authored,
        "every <Key/> the loops emitted mounted the registered component"
    );

    // Its output is a subtree this crate never authored, so the rendered text is
    // the only evidence the component ran rather than the tag becoming a box.
    let rendered: Vec<&str> = nodes.iter().map(|node| node.text.as_str()).collect();
    assert!(rendered.contains(&"shift"), "a per-row label reached its key: {rendered:?}");
    assert!(
        rendered.contains(&"wide"),
        "and so did a derived prop the loop computed: {rendered:?}"
    );
}

#[test]
fn an_unregistered_component_is_the_silence_this_fixture_guards_against() {
    // The control: the same markup with no registration mounts nothing, renders
    // no key text, and reports no error. That silence is what made audit 1-A read
    // as "the keyboard draws its shell and no keys", so it is worth pinning what
    // it looks like — a passing build is not evidence a component is live.
    let mut runtime = runtime(640.0, 320.0);
    let nodes = measure_in(&mut runtime, &keyboard_view(&sample_rows()), CSS);
    assert_eq!(runtime.mounted_component_count(), 0);
    let rendered: Vec<&str> = nodes.iter().map(|node| node.text.as_str()).collect();
    assert!(!rendered.contains(&"shift"), "{rendered:?}");
    // The shell is still there, which is exactly why the failure is quiet.
    assert!(find(&nodes, "keyboard").is_some());
}

#[test]
fn the_name_a_positional_binding_targets_resolves_against_the_tree() {
    // `on_command_at` in `panel.rs` targets this name. It can only bind once a
    // tree exists, which is why the layer loop has to apply the control queue
    // AFTER a frame commits and not only when something is queued. Whether the
    // loop does that is not testable without a compositor; that the name is a
    // real one is, and a name that resolved to nothing would make the question
    // moot.
    let mut runtime = runtime(640.0, 320.0);
    register_key(&mut runtime);
    let _ = measure_in(&mut runtime, &keyboard_view(&sample_rows()), CSS);

    let resolver = SelectorResolver::new();
    assert!(
        resolver
            .resolve(&runtime, &SelectorTarget::test_id(KEYBOARD_TEST_ID))
            .is_some(),
        "the `test-id` the markup carries names a live node"
    );
    assert_eq!(
        resolver.resolve(&runtime, &SelectorTarget::test_id("no-such-root")),
        None,
        "and the resolver is not simply saying yes"
    );
}

#[test]
fn asking_which_displays_exist_answers_rather_than_panicking() {
    // Runs wherever it runs — a compositor, a CI container, an X11 session — and
    // the shape of the answer is the assertion. Before `App::displays` existed
    // there was no way to reach an id at all before `run`, so an application had
    // to pass a position in the compositor's discovery order.
    match choose_display("eDP-1") {
        Ok(DisplayChoice::Chosen(id)) => assert!(id.raw() >= 1, "ids are handed out from one"),
        Ok(DisplayChoice::NoSuchDisplay(names)) => {
            assert!(
                !names.iter().any(|name| name == "eDP-1"),
                "it said no such display, so it must not be listing one: {names:?}"
            );
        }
        Ok(DisplayChoice::NoDesktop) => {}
        Err(error) => panic!("a query must answer, not fail opaquely: {error}"),
    }
}

#[test]
fn an_empty_display_list_has_no_first_display() {
    assert_eq!(first_display(&[]), None);
}
