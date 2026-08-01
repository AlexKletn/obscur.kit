#![doc = include_str!("spec_ch3_transactions.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch3_support/mod.rs"]
mod support;

use obscur_kit::runtime::transaction::{TransactionValidator, UiOperation, UiTransaction};
use support::{approx, at, drain_frames, el, geom, runtime, view};

#[doc = include_str!("spec_ch3_transactions.doc/a_batch_with_one_invalid_op_applies_none_of_it.md")]
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

    let mut tx = UiTransaction::new();
    tx.push(UiOperation::InvalidateLayout { node: node_a }); // op1: valid
    tx.push(UiOperation::InvalidateLayout { node: node_a }); // op2: invalid — same node, same kind

    let outcome = TransactionValidator::new().validate(&tx);
    let error = outcome.expect_err(
        "the commit must return a typed validation failure — a batch is committed \
         atomically, so one bad op rejects the whole batch",
    );
    assert!(
        error.to_string().contains(&node_a.raw().to_string()),
        "the failure names op2's target ({}), got: {error}",
        node_a.raw()
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

#[doc = include_str!("spec_ch3_transactions.doc/a_restored_scroll_offset_is_clamped_to_the_new_content_extent.md")]
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
