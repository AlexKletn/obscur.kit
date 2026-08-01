#![doc = include_str!("virtual_realization.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_geometry::Size;
use obscur_runtime::UiRuntime;
use obscur_testkit::fixture::{TestFixture, VirtualRowList};
use obscur_testkit::selector::{NodePath, SemanticSelector};

/// The window a `<VirtualFor>` realizes is bounded by the viewport, not by the
/// item count.
#[test]
fn virtual_for_realizes_a_viewport_bounded_window_not_the_item_count() {
    // §4.9: a `<VirtualFor>` marker is realized into a window whose size comes
    // from `VirtualLayout` against the committed viewport — tens of rows, not
    // the (assumed-huge) item count. A taller viewport realizes strictly more,
    // which is what proves the bound is the viewport and not a constant.
    let mut short = UiRuntime::new(TestFixture::RUNTIME_ID, Size::new(300.0, 400.0));
    short.render_now(&VirtualRowList::view(), &[]);
    let short_rows = SemanticSelector::count_class(&short, "row");

    let mut tall = UiRuntime::new(TestFixture::RUNTIME_ID, Size::new(300.0, 1600.0));
    tall.render_now(&VirtualRowList::view(), &[]);
    let tall_rows = SemanticSelector::count_class(&tall, "row");

    // 400px / 24px + overscan is on the order of tens, and far below the
    // million-item collection the window is sized against.
    assert!(
        (1..200).contains(&short_rows),
        "a 400px viewport realizes tens of rows, got {short_rows}"
    );
    assert!(
        tall_rows > short_rows,
        "a 4x-taller viewport realizes strictly more rows ({tall_rows} vs {short_rows}), \
         so the bound is the viewport"
    );
    // The marker itself is substituted away: the realized rows are the direct
    // children of the div, with no virtual-collection node retained between.
    let div = NodePath::new(&[0]).resolve(&short);
    assert_eq!(
        short.tree().get(div).expect("live div").child_nodes().len(),
        short_rows,
        "every realized row is a direct child of the div; no marker node remains"
    );
}
