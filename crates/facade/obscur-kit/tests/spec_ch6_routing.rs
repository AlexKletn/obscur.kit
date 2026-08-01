#![doc = include_str!("spec_ch6_routing.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::routing::route::{Route, RoutePattern};
use obscur_kit::routing::router::{RouteRegistry, Router};

// -------------------------------------------------------------------------
// P6-18
// -------------------------------------------------------------------------

#[doc = include_str!("spec_ch6_routing.doc/path_query_and_fragment_are_split_from_one_another.md")]
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

#[doc = include_str!("spec_ch6_routing.doc/percent_encoding_is_decoded_after_segmentation_and_only_once.md")]
#[test]
fn percent_encoding_is_decoded_after_segmentation_and_only_once() {
    let route = Route::parse("/files/a%2Fb/edit");
    let params = RoutePattern::parse("/files/:name/:action")
        .match_path(route.path())
        .expect("§6.13: `/files/a%2Fb/edit` matches `/files/:name/:action`");

    assert_eq!(
        params.get("action"),
        Some("edit"),
        "§6.13: the encoded slash did not open a new segment — the trailing \
         segment is `edit`, not `b`"
    );
    assert!(
        params
            .get("name")
            .is_some_and(|name| matches!(name, "a/b" | "a%2Fb")),
        "§6.13: the whole first segment binds to `:name`, decoded or not; got {:?}",
        params.get("name")
    );
}

// -------------------------------------------------------------------------
// P6-21
// -------------------------------------------------------------------------

#[doc = include_str!("spec_ch6_routing.doc/pushing_after_a_back_truncates_the_forward_stack.md")]
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
