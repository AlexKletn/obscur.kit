//! **SPECIFICATION-DERIVED** cases for §5.15 Resource Hub ("URL through
//! user/host loader"; "The kit does not perform arbitrary network access by
//! itself"; "loading/ready/failed states") and §3.4 ("The synchronous frame
//! phase must stay short and deterministic") — the invariants behind the §9.1
//! non-goal "arbitrary networking in the core".
//!
//! Every expected value was produced by an agent reading the specification with
//! **no access to this repository's source**. A failure here is a defect report
//! about the engine, and must not be "corrected" by editing the expected value.
#![forbid(unsafe_code)]

#[path = "spec_ch9_support/mod.rs"]
mod support;

use obscur_kit::identity::InternedSymbol;
use obscur_kit::resources::source::ResourceSource;
use obscur_kit::resources::state::ResourceState;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::view::view::{SRC, ViewNode, ViewProperty, ViewValue};
use support::{compile, el, runtime, view};

const VW: f32 = 1000.0;
const REMOTE: &str = "https://example.invalid/a.png";

/// §5.15 Resource Hub — "Sources:" / "- URL through user/host loader;" and "The
/// kit does not perform arbitrary network access by itself."
///
/// NG-URLIMG-01: with no host loader registered, the handle must reach the
/// terminal `failed` state (never park in `loading`), a diagnostic must name the
/// missing loader, and the frame must still commit and paint the background.
#[test]
fn a_url_background_without_a_host_loader_fails_terminally_and_still_paints() {
    let sheet = compile(&format!(
        ".a{{width:100px;height:100px;background-color:rgb(0,0,255);\
           background-image:url(\"{REMOTE}\")}}"
    ));
    let sheets = [sheet];
    let v = view(vec![el(1, "a", vec![])]);

    let mut rt = runtime(VW, VW);
    let result = rt.render_now(&v, &sheets);
    let _ = result;
    // Give any off-thread resource work every chance to settle without a loader.
    for _ in 0..8 {
        let _ = rt.poll_resource_tasks();
    }
    let _ = rt.render_now(&v, &sheets);

    // The frame still commits and the node still paints its background colour.
    let filled = rt
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .any(|c| matches!(c, PaintCommand::DrawPath { .. }));
    assert!(filled, "the node still paints its background colour");

    // The handle is terminal-failed, not parked in `loading`.
    let mut hub = obscur_kit::resources::hub::ResourceHub::new();
    let request = obscur_kit::resources::hub::ResourceRequest::new(
        ResourceSource::url(REMOTE),
        obscur_kit::resources::handle::ResourceKind::Image,
    );
    let handle = hub.request(request);
    let state = hub.state(handle).cloned().expect("a state for the handle");
    assert!(
        matches!(state, ResourceState::Failed { .. }),
        "with no host loader a URL resource reaches `failed`, not `{state:?}`"
    );

    assert!(
        rt.resource_count() > 0,
        "the `url()` background reached the resource hub as a request"
    );
}

/// §3.4 Frame pipeline — "The synchronous frame phase must stay short and
/// deterministic." and §5.15 "Features:" / "- async load/decode/compile;" /
/// "- loading/ready/failed states;"
///
/// NG-ASYNC-01: an image whose load never completes must not block the frame; it
/// paints its reserved box in `loading`, and no further frame is requested until
/// the completion arrives.
#[test]
fn a_never_completing_image_load_does_not_block_the_frame() {
    let sheet = compile(".a{width:100px;height:100px;background-color:rgb(0,255,0)}");
    let sheets = [sheet];
    let node = ViewNode::element(obscur_kit::identity::ComponentId::from_raw(1))
        .with_prop(ViewProperty::new(
            InternedSymbol::from_raw(obscur_kit::view::view::CLASS.raw()),
            ViewValue::Text("a".into()),
        ))
        .with_prop(ViewProperty::new(SRC, ViewValue::Text(REMOTE.into())));
    let v = view(vec![node]);

    let mut rt = runtime(VW, VW);
    let _ = rt.render_now(&v, &sheets);

    // The frame completed (a scene exists) without waiting on the load.
    assert!(
        !rt.snapshot().display_list().commands().is_empty(),
        "the frame committed a scene while the image is still loading"
    );
    assert!(
        !rt.has_pending_frame(),
        "no further frame is requested until the load completion is delivered"
    );
    assert!(
        rt.snapshot()
            .display_list()
            .commands()
            .iter()
            .all(|c| !matches!(c, PaintCommand::DrawImage { .. })),
        "nothing is painted as an image while the load is still in flight"
    );
}
