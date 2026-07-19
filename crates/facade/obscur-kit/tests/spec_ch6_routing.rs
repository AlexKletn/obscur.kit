//! **SPECIFICATION-DERIVED** router cases for obscur.kit §6.13.
//!
//! Every expected value below was produced by an agent reading obscur.kit §6
//! with **no access to this repository's source**. A failure here is a defect
//! report about the toolkit, not about the test; expected values must not be
//! edited to match current behaviour.
//!
//! Each test's doc comment carries the quoted normative sentence it encodes.
#![forbid(unsafe_code)]

use obscur_kit::routing::route::{Route, RoutePattern};
use obscur_kit::routing::router::{RouteRegistry, Router};

// -------------------------------------------------------------------------
// P6-18
// -------------------------------------------------------------------------

/// §6.13 Router: "Basic URL-like router:" / "- path;" / "- query;" /
/// "- fragment;"
///
/// Path, query and fragment are split from one another before any of them is
/// interpreted: no `?` residue in the path, no `#` residue in the last query
/// value, and the fragment carries no leading `#`.
#[test]
fn path_query_and_fragment_are_split_from_one_another() {
    let route = Route::parse("/users/42?tab=posts&page=2#comments");

    assert_eq!(
        route.segments(),
        vec!["users", "42"],
        "§6.13: the path is `/users/42` with no query or fragment residue"
    );
    assert_eq!(
        route.query().len(),
        2,
        "§6.13: exactly two query entries, got {:?}",
        route.query()
    );
    assert_eq!(
        route.query_param("tab"),
        Some("posts"),
        "§6.13: query `tab`"
    );
    assert_eq!(
        route.query_param("page"),
        Some("2"),
        "§6.13: the fragment is not part of the last query value"
    );
    assert_eq!(
        route.fragment(),
        Some("comments"),
        "§6.13: the fragment is reported without its leading `#`"
    );

    let params = RoutePattern::parse("/users/:id")
        .match_path(route.path())
        .expect("§6.13: `/users/42` matches `/users/:id`");
    assert_eq!(
        params.get("id"),
        Some("42"),
        "§6.13: the `id` param carries no `?`/`#` residue"
    );
}

// -------------------------------------------------------------------------
// P6-19
// -------------------------------------------------------------------------

/// §6.13: "- path;" (URL-like) with "- typed params;"
///
/// Percent-decoding happens **after** segmentation and exactly once, so an
/// encoded slash stays inside one segment.
#[test]
fn percent_encoding_is_decoded_after_segmentation_and_only_once() {
    let route = Route::parse("/files/a%2Fb/edit");
    let params = RoutePattern::parse("/files/:name/:action")
        .match_path(route.path())
        .expect("§6.13: `/files/a%2Fb/edit` matches `/files/:name/:action`");

    assert_eq!(
        params.get("name"),
        Some("a/b"),
        "§6.13: `%2F` is decoded once, after segmentation — not into a segment break"
    );
    assert_eq!(
        params.get("action"),
        Some("edit"),
        "§6.13: the trailing segment is `edit`, not `b`"
    );

    let double = Route::parse("/files/a%252Fb/edit");
    let double_params = RoutePattern::parse("/files/:name/:action")
        .match_path(double.path())
        .expect("§6.13: the double-encoded path matches too");
    assert_eq!(
        double_params.get("name"),
        Some("a%2Fb"),
        "§6.13: decoding happens exactly once, never twice"
    );
}

// -------------------------------------------------------------------------
// P6-21
// -------------------------------------------------------------------------

/// §6.13: "- history;" / "- back/forward;"
///
/// Pushing after a back discards the forward future, and backing past the first
/// entry is a typed no-op rather than an underflow.
#[test]
fn pushing_after_a_back_truncates_the_forward_stack() {
    let registry = RouteRegistry::new()
        .with_route(RoutePattern::parse("/a"), "a")
        .with_route(RoutePattern::parse("/b"), "b")
        .with_route(RoutePattern::parse("/c"), "c")
        .with_route(RoutePattern::parse("/d"), "d");
    let mut router = Router::new(registry);

    router.navigate("/a");
    router.navigate("/b");
    router.navigate("/c");

    router.back().expect("§6.13: a back from /c reaches /b");
    assert_eq!(
        router.current().map(|e| e.route().path()),
        Some("/b"),
        "§6.13: back from /c lands on /b"
    );
    assert!(
        router.can_go_forward(),
        "§6.13: /c is still reachable forward"
    );

    router.navigate("/d");
    assert_eq!(
        router.current().map(|e| e.route().path()),
        Some("/d"),
        "§6.13: the push lands on /d"
    );
    assert!(
        !router.can_go_forward(),
        "§6.13: pushing discards the /c future — forward must not teleport there"
    );

    // Four backs from /d: the stack behind it is [/a, /b].
    router.back();
    router.back();
    let third = router.back();
    let fourth = router.back();
    assert_eq!(
        router.current().map(|e| e.route().path()),
        Some("/a"),
        "§6.13: backing past the first entry stays on /a — no wrap, no underflow"
    );
    assert!(
        third.is_none() && fourth.is_none(),
        "§6.13: a back with no earlier entry is a typed no-op the caller can read"
    );
}
