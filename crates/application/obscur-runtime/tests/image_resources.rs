#![doc = include_str!("image_resources.doc/_module.md")]
#![forbid(unsafe_code)]

use std::sync::Arc;

use obscur_runtime::frame::FrameReason;
use obscur_testkit::assertion::PaintedImages;
use obscur_testkit::fixture::{CounterPanel, ImagePanel, TestFixture, render_resolved};
use obscur_testkit::mock::{CannedImageDecoder, CannedImageLoader, QueuedExecutor};

/// A `background-image: url(…)` that resolves paints exactly one image.
#[test]
fn url_background_image_resolves_and_paints_a_draw_image() {
    let mut runtime = TestFixture::runtime();
    runtime.set_resource_providers(
        Arc::new(CannedImageLoader::working()),
        Arc::new(CannedImageDecoder),
    );

    let sheet = ImagePanel::sheet();
    let result = render_resolved(&mut runtime, &ImagePanel::view(), &sheet);
    let list = result.snapshot().display_list();
    assert_eq!(
        PaintedImages::count(list),
        1,
        "the resolved url() image paints one DrawImage"
    );
    assert!(result.is_balanced());
}

/// An element's own `src` travels the same path and paints one image too.
#[test]
fn element_image_src_resolves_and_paints_a_draw_image() {
    // A `<img>` element's own `src` loads + decodes into the cache (same path as
    // background images) and paints one DrawImage over its 100×50 box.
    let mut runtime = TestFixture::runtime();
    runtime.set_resource_providers(
        Arc::new(CannedImageLoader::working()),
        Arc::new(CannedImageDecoder),
    );
    let sheet = CounterPanel::sheet();
    let result = render_resolved(&mut runtime, &ImagePanel::src_view("logo"), &sheet);
    assert_eq!(
        PaintedImages::count(result.snapshot().display_list()),
        1,
        "the element image resolves and paints one DrawImage"
    );
    assert!(result.is_balanced());
}

/// A failed element load paints nothing, and does not take the frame with it.
#[test]
fn element_image_loader_error_paints_no_image_and_does_not_panic() {
    let mut runtime = TestFixture::runtime();
    runtime.set_resource_providers(
        Arc::new(CannedImageLoader::failing()),
        Arc::new(CannedImageDecoder),
    );
    let sheet = CounterPanel::sheet();
    let result = runtime.render_now(&ImagePanel::src_view("logo"), std::slice::from_ref(&sheet));
    assert_eq!(
        PaintedImages::count(result.snapshot().display_list()),
        0,
        "a failed load caches nothing → no element DrawImage"
    );
    assert!(result.is_balanced());
}

/// With no providers installed there is no image, and that is the honest answer.
#[test]
fn url_background_image_with_no_providers_paints_no_image() {
    // Honest-capability default: no loader/decoder installed → no image.
    let mut runtime = TestFixture::runtime();
    let sheet = ImagePanel::sheet();
    let result = runtime.render_now(&ImagePanel::view(), std::slice::from_ref(&sheet));
    assert_eq!(PaintedImages::count(result.snapshot().display_list()), 0);
}

/// A failed `url(…)` load paints nothing either.
#[test]
fn url_background_image_loader_error_paints_no_image_and_does_not_panic() {
    let mut runtime = TestFixture::runtime();
    runtime.set_resource_providers(
        Arc::new(CannedImageLoader::failing()),
        Arc::new(CannedImageDecoder),
    );

    let sheet = ImagePanel::sheet();
    let result = runtime.render_now(&ImagePanel::view(), std::slice::from_ref(&sheet));
    assert_eq!(
        PaintedImages::count(result.snapshot().display_list()),
        0,
        "a failed load caches nothing → no DrawImage"
    );
    assert!(result.is_balanced());
}

/// Without a blocking executor the load still happens — later, off the frame.
#[test]
fn with_no_executor_a_load_still_runs_on_the_frame_thread() {
    // The capability default is not a degradation: without the port the load
    // still runs on the frame thread and spawns no background thread. What
    // moved is its TIMING — it is deferred out of the synchronous frame phase
    // (§3.4, §5.15), so the first frame paints without the image, `poll` runs
    // the queued load inline, and the next frame composes it.
    let mut runtime = TestFixture::runtime();
    runtime.set_resource_providers(
        Arc::new(CannedImageLoader::working()),
        Arc::new(CannedImageDecoder),
    );

    let sheet = ImagePanel::sheet();
    let first = runtime.render_now(&ImagePanel::view(), std::slice::from_ref(&sheet));
    assert_eq!(
        PaintedImages::count(first.snapshot().display_list()),
        0,
        "the load is deferred out of the frame phase — nothing paints yet"
    );
    assert_eq!(
        runtime.poll_resource_tasks(),
        1,
        "the deferred load runs on poll — on the frame thread, nothing spawned"
    );
    let second = runtime.render_now(&ImagePanel::view(), std::slice::from_ref(&sheet));
    assert_eq!(
        PaintedImages::count(second.snapshot().display_list()),
        1,
        "the re-render composes the now-decoded image"
    );
}

/// A completion the frame commits must also DIRTY that frame, or committing
/// it would destroy it.
///
/// The trap this pins: an idle frame (a diagnostics capture recalculates
/// nothing and re-presents the retained scene) still drains the channel, so
/// without the plan merge the result is consumed and thrown away — the image
/// would never paint, because nothing would ever request a frame for it
/// again. `DiagnosticsCapture` is the only reason that maps to an idle plan,
/// which is why it is the reason used here.
#[test]
fn a_completion_committed_on_an_idle_frame_still_paints() {
    let executor = Arc::new(QueuedExecutor::default());
    let mut runtime = TestFixture::runtime();
    runtime.set_resource_providers(
        Arc::new(CannedImageLoader::working()),
        Arc::new(CannedImageDecoder),
    );
    runtime.set_blocking_executor(executor.clone());

    let sheet = ImagePanel::sheet();
    // First frame builds the tree and hands the load off.
    runtime.render_now(&ImagePanel::view(), std::slice::from_ref(&sheet));
    executor.run_all();

    // An idle frame, and nothing polled: the result is waiting on the channel.
    runtime.request_frame(FrameReason::DiagnosticsCapture);
    let idle = runtime
        .run_due_frame(&ImagePanel::view(), std::slice::from_ref(&sheet))
        .expect("the capture frame is due");
    assert_eq!(
        PaintedImages::count(idle.snapshot().display_list()),
        1,
        "the frame that consumed the completion must be dirtied by it"
    );
}

/// A failure is drained but caches nothing, so it asks for no frame.
#[test]
fn a_failed_load_is_applied_as_a_failure_and_paints_nothing() {
    let executor = Arc::new(QueuedExecutor::default());
    let mut runtime = TestFixture::runtime();
    runtime.set_resource_providers(
        Arc::new(CannedImageLoader::failing()),
        Arc::new(CannedImageDecoder),
    );
    runtime.set_blocking_executor(executor.clone());

    let sheet = ImagePanel::sheet();
    runtime.render_now(&ImagePanel::view(), std::slice::from_ref(&sheet));
    executor.run_all();
    // A failure is drained but not "applied": it caches nothing, so no frame is
    // requested for it either.
    assert_eq!(runtime.poll_resource_tasks(), 0);
    let list = runtime
        .render_now(&ImagePanel::view(), std::slice::from_ref(&sheet))
        .snapshot()
        .display_list()
        .clone();
    assert_eq!(
        PaintedImages::count(&list),
        0,
        "a failed load paints nothing"
    );
}
