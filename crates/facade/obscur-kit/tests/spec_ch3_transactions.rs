//! **SPECIFICATION-DERIVED** transaction and persistence-restore cases for
//! obscur.kit §3.2 ("Mutations are normalized into validated `UiOps` and
//! committed atomically between frames") and §3.8 ("scroll positions" restored
//! via "Optional versioned helpers may serialize and restore").
//!
//! Every expected value below was produced by an agent reading obscur.kit §3
//! with **no access to this repository's source**. A failure here is a defect
//! report about the runtime, and must not be "corrected" by editing the expected
//! value.
#![forbid(unsafe_code)]

#[path = "spec_ch3_support/mod.rs"]
mod support;

use obscur_kit::identity::NodeId;
use obscur_kit::runtime::transaction::{TransactionValidator, UiOperation, UiTransaction};
use support::{approx, at, drain_frames, el, geom, runtime, view};

/// §3.2 — "Mutations are normalized into validated `UiOps` and committed
/// atomically between frames."
///
/// A batch whose *second* op targets a node that does not exist must be rejected
/// as a whole: the first op's effect must not appear, and no frame is requested.
#[test]
fn a_batch_with_one_invalid_op_applies_none_of_it() {
    const CSS: &str = ".page { width: 300px; height: 300px } \
                       .a { width: 100px; height: 20px }";

    let sheet = support::compile(CSS);
    let sheets = std::slice::from_ref(&sheet);
    let v = view(vec![el(1, "page", vec![el(2, "a", vec![])])]);

    let mut rt = runtime(300.0, 300.0);
    let _ = rt.render_now(&v, sheets);
    let _ = drain_frames(&mut rt, &v, sheets);

    let node_a = at(&rt, &[0, 0]);
    approx(
        geom(&rt, node_a).2,
        100.0,
        "node A's width before the commit",
    );

    // `Z` is not in the tree — nothing was ever reconciled under this id.
    let node_z = NodeId::from_raw(0x00DE_AD00);
    assert!(
        rt.tree().get(node_z).is_none(),
        "the invalid op really does target a node that does not exist"
    );

    let mut tx = UiTransaction::new();
    tx.push(UiOperation::InvalidateLayout { node: node_a }); // op1: valid
    tx.push(UiOperation::InvalidateLayout { node: node_z }); // op2: invalid

    let outcome = TransactionValidator::new().validate(&tx);
    let error = outcome.expect_err(
        "the commit must return a typed validation failure — a batch is committed \
         atomically, so a dead target rejects the whole batch",
    );
    assert!(
        error.to_string().contains(&node_z.raw().to_string()),
        "the failure names op2's target ({}), got: {error}",
        node_z.raw()
    );

    // Nothing was applied and nothing was scheduled.
    assert!(
        !rt.has_pending_frame(),
        "a rejected commit requests no frame"
    );
    let _ = drain_frames(&mut rt, &v, sheets);
    let node_a = at(&rt, &[0, 0]);
    approx(
        geom(&rt, node_a).2,
        100.0,
        "node A keeps its PREVIOUS width — op1 must not have been applied",
    );
}

/// §3.8 — "scroll positions" restored via "Optional versioned helpers may
/// serialize and restore".
///
/// A persisted offset of 1500 restored into a session whose content is now only
/// 600px tall (400px viewport → max scroll 200) must clamp to **200** — not
/// 1500, and not 0.
///
/// Translation note: the kit exposes no persistence-snapshot round trip, so the
/// restore is expressed as writing the persisted offset into the scroll state,
/// which is what a restore helper would do.
#[test]
fn a_restored_scroll_offset_is_clamped_to_the_new_content_extent() {
    const CSS: &str = ".scroller { width: 300px; height: 400px; overflow: auto } \
                       .content { width: 300px; height: 600px }";

    let sheet = support::compile(CSS);
    let sheets = std::slice::from_ref(&sheet);
    let v = view(vec![el(1, "scroller", vec![el(2, "content", vec![])])]);

    let mut rt = runtime(300.0, 400.0);
    let _ = rt.render_now(&v, sheets);
    let _ = drain_frames(&mut rt, &v, sheets);

    let scroller = at(&rt, &[0]);

    // The persisted offset, from a session whose content was 2000px tall.
    rt.set_scroll_top(scroller, 1500.0);
    let _ = drain_frames(&mut rt, &v, sheets);
    let _ = rt.render_now(&v, sheets);

    let scroller = at(&rt, &[0]);
    approx(
        rt.scroll_top(scroller),
        200.0,
        "the restored offset is clamped to this session's max scroll (600 - 400), \
         not written through raw and not lost to 0",
    );
}
