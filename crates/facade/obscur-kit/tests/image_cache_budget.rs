#![doc = include_str!("image_cache_budget.doc/_module.md")]
#![forbid(unsafe_code)]
#![cfg(all(feature = "resource-fs", feature = "resource-decode"))]

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use obscur_kit::bootstrap::KitBootstrap;
use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::devtools::inspect_source_from;
use obscur_kit::devtools_protocol::message::Inspectable;
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::primitives::Image;
use obscur_kit::resources::cache::CacheBudget;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewProperty, ViewValue};

#[doc = include_str!("image_cache_budget.doc/VIEWPORT.md")]
const VIEWPORT: f32 = 64.0;

#[doc = include_str!("image_cache_budget.doc/RED_2X2_PNG.md")]
const RED_2X2_PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 2, 0, 0, 0, 2, 8, 2, 0,
    0, 0, 253, 212, 154, 115, 0, 0, 0, 16, 73, 68, 65, 84, 120, 218, 99, 248, 207, 192, 0, 68, 12,
    16, 10, 0, 31, 238, 3, 253, 99, 94, 187, 91, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

#[doc = include_str!("image_cache_budget.doc/IMAGE_BYTES.md")]
const IMAGE_BYTES: u64 = 2 * 2 * 4;

#[doc = include_str!("image_cache_budget.doc/COUNTER.md")]
static COUNTER: AtomicU32 = AtomicU32::new(0);

#[doc = include_str!("image_cache_budget.doc/temp_asset_dir.md")]
fn temp_asset_dir() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "obscur-kit-cache-budget-{}-{n}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    for name in ["a.png", "b.png", "c.png"] {
        std::fs::write(dir.join(name), RED_2X2_PNG).expect("write png");
    }
    dir
}

#[doc = include_str!("image_cache_budget.doc/compile.md")]
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("image_cache_budget.doc/img.md")]
fn img(src: &str, class: &str) -> obscur_kit::view::view::ViewNode {
    Image::new(src)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

#[doc = include_str!("image_cache_budget.doc/runtime_at.md")]
fn runtime_at(dir: &PathBuf) -> UiRuntime {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    KitBootstrap::new().install_image_resources(&mut runtime, dir);
    runtime
}

#[doc = include_str!("image_cache_budget.doc/painted_at_y.md")]
fn painted_at_y(result: &obscur_kit::runtime::frame::FrameResult, y: f32) -> bool {
    result
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .any(|command| match command {
            PaintCommand::DrawImage { region, .. } => (region.origin.y - y).abs() < 0.5,
            _ => false,
        })
}

#[doc = include_str!("image_cache_budget.doc/a_byte_budget_evicts_while_composing_a_real_frame.md")]
#[test]
fn a_byte_budget_evicts_while_composing_a_real_frame() {
    let dir = temp_asset_dir();
    let sheet = compile(
        ".a { position: absolute; left: 0; top: 0; width: 4px; height: 4px }
         .b { position: absolute; left: 0; top: 8px; width: 4px; height: 4px }
         .c { position: absolute; left: 0; top: 16px; width: 4px; height: 4px }",
    );
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        img("a.png", "a"),
        img("b.png", "b"),
        img("c.png", "c"),
    ]));

    let mut runtime = runtime_at(&dir);
    // Room for two of the three 16-byte images.
    runtime.set_image_cache_budget(CacheBudget::bytes(2 * IMAGE_BYTES));
    runtime.render_now(&view, std::slice::from_ref(&sheet));
    // The load is deferred out of the frame phase (no blocking executor): draining
    // it here runs the load, admits the pixels, and lets the budget evict — the
    // same insert-time eviction the old synchronous frame did in-line.
    let _ = runtime.poll_resource_tasks();

    let stats = runtime.image_cache_statistics();
    assert_eq!(
        stats.entries(),
        2,
        "three images were referenced but only two fit the budget"
    );
    assert!(
        stats.bytes() <= 2 * IMAGE_BYTES,
        "resident bytes must stay within the budget, got {}",
        stats.bytes()
    );
    assert_eq!(
        stats.evictions(),
        1,
        "seating the third evicted exactly one"
    );
    assert_eq!(stats.insertions(), 3, "all three decoded and were admitted");
    assert_eq!(
        runtime.cached_resource_counts().0,
        2,
        "the DevTools resident count agrees with the metrics"
    );
}

#[doc = include_str!("image_cache_budget.doc/the_devtools_caches_inspectable_reports_what_the_budget_did.md")]
#[test]
fn the_devtools_caches_inspectable_reports_what_the_budget_did() {
    let dir = temp_asset_dir();
    let sheet = compile(
        ".a { position: absolute; left: 0; top: 0; width: 4px; height: 4px }
         .b { position: absolute; left: 0; top: 8px; width: 4px; height: 4px }
         .c { position: absolute; left: 0; top: 16px; width: 4px; height: 4px }",
    );
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        img("a.png", "a"),
        img("b.png", "b"),
        img("c.png", "c"),
    ]));

    let mut runtime = runtime_at(&dir);
    runtime.set_image_cache_budget(CacheBudget::bytes(2 * IMAGE_BYTES));
    runtime.render_now(&view, std::slice::from_ref(&sheet));
    // Drive the deferred load so the budget's insert-time eviction actually runs
    // before we read what the DevTools inspectable reports.
    let _ = runtime.poll_resource_tasks();

    let source = inspect_source_from(&runtime);
    let payload = source
        .registered(Inspectable::Caches)
        .expect("the Caches inspectable is registered");
    let detail = &payload.detail;

    assert!(
        detail.contains("image_cache evictions: 1"),
        "the eviction the budget forced never reached a DevTools client: {detail}"
    );
    assert!(
        detail.contains(&format!("({} bytes)", 2 * IMAGE_BYTES)),
        "the resident byte total never reached a DevTools client: {detail}"
    );
}

#[doc = include_str!("image_cache_budget.doc/the_default_budget_is_unlimited_and_never_evicts.md")]
#[test]
fn the_default_budget_is_unlimited_and_never_evicts() {
    let dir = temp_asset_dir();
    let sheet = compile(
        ".a { position: absolute; left: 0; top: 0; width: 4px; height: 4px }
         .b { position: absolute; left: 0; top: 8px; width: 4px; height: 4px }
         .c { position: absolute; left: 0; top: 16px; width: 4px; height: 4px }",
    );
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        img("a.png", "a"),
        img("b.png", "b"),
        img("c.png", "c"),
    ]));

    let mut runtime = runtime_at(&dir);

    // The first compose finds nothing resident — three misses — and, with no
    // blocking executor, the loads are deferred out of the frame phase (§3.3/§3.4),
    // so nothing is resident or painted yet. The miss signal is recorded here.
    runtime.render_now(&view, std::slice::from_ref(&sheet));
    let stats = runtime.image_cache_statistics();
    assert_eq!(stats.hits(), 0, "the first frame could only miss");
    assert_eq!(stats.misses(), 3, "one miss per referenced URL");

    // Drive the deferred loads (on the frame thread, after the frame phase), then
    // compose again so every decoded image is resident and paints.
    let _ = runtime.poll_resource_tasks();
    let result = runtime.render_now(&view, std::slice::from_ref(&sheet));

    let stats = runtime.image_cache_statistics();
    assert_eq!(stats.entries(), 3, "every image stays resident");
    assert_eq!(stats.evictions(), 0, "an unlimited budget evicts nothing");
    assert_eq!(stats.bytes(), 3 * IMAGE_BYTES);
    for y in [0.0, 8.0, 16.0] {
        assert!(painted_at_y(&result, y), "all three images paint at y={y}");
    }

    // The first frame found nothing resident and had to load all three; the second
    // finds every one of them and loads nothing. That is the hit/miss signal §5.19
    // asks DevTools to be able to see, and it is recorded by composing a frame —
    // not by the test.
    assert_eq!(stats.hits(), 3, "the second frame hits every cached image");
    assert_eq!(stats.misses(), 3, "and misses nothing new");
    assert_eq!(stats.insertions(), 3, "nothing was decoded twice");
    assert!((stats.hit_rate() - 0.5).abs() < 1e-6, "3 hits of 6 lookups");
}

#[doc = include_str!("image_cache_budget.doc/eviction_takes_the_offscreen_image_not_the_visible_one.md")]
#[test]
fn eviction_takes_the_offscreen_image_not_the_visible_one() {
    let dir = temp_asset_dir();
    let sheet = compile(
        ".a { position: absolute; left: 0; top: 0; width: 4px; height: 4px }
         .off { position: absolute; left: 0; top: 5000px; width: 4px; height: 4px }
         .c { position: absolute; left: 0; top: 16px; width: 4px; height: 4px }",
    );
    // Document order fixes the touch order: a (visible) before b (off-screen).
    let two = View::from_fragment(ViewFragment::from_nodes(vec![
        img("a.png", "a"),
        img("b.png", "off"),
    ]));
    let three = View::from_fragment(ViewFragment::from_nodes(vec![
        img("a.png", "a"),
        img("b.png", "off"),
        img("c.png", "c"),
    ]));

    let mut runtime = runtime_at(&dir);
    // Exactly two images fit, so introducing a third forces one choice.
    runtime.set_image_cache_budget(CacheBudget::bytes(2 * IMAGE_BYTES));

    // Frame 1: both load and both fit — nothing is evicted yet.
    runtime.render_now(&two, std::slice::from_ref(&sheet));
    // The two loads are deferred out of the frame phase; drain them so both are
    // resident (and their visibility stamped) before the third frame forces a choice.
    let _ = runtime.poll_resource_tasks();
    assert_eq!(runtime.image_cache_statistics().entries(), 2);
    assert_eq!(runtime.image_cache_statistics().evictions(), 0);

    // Frame 2: a third on-screen image arrives and one of the two must go.
    // Render (queues c's load, stamps this frame's visibility), drain (admits c,
    // which over-fills the budget and evicts the off-screen victim), then render
    // again so the composed scene reflects who survived.
    runtime.render_now(&three, std::slice::from_ref(&sheet));
    let _ = runtime.poll_resource_tasks();
    let result = runtime.render_now(&three, std::slice::from_ref(&sheet));

    assert_eq!(runtime.image_cache_statistics().evictions(), 1);
    assert!(
        painted_at_y(&result, 0.0),
        "the VISIBLE a.png must survive: it is what the frame is showing"
    );
    assert!(painted_at_y(&result, 16.0), "the newly loaded c.png paints");
    assert!(
        !painted_at_y(&result, 5000.0),
        "the off-screen b.png is the victim"
    );
}

#[doc = include_str!("image_cache_budget.doc/an_evicted_image_reloads_when_referenced_again.md")]
#[test]
fn an_evicted_image_reloads_when_referenced_again() {
    let dir = temp_asset_dir();
    let sheet = compile(".a { position: absolute; left: 0; top: 0; width: 4px; height: 4px }");
    let view = View::from_fragment(ViewFragment::from_nodes(vec![img("a.png", "a")]));

    let mut runtime = runtime_at(&dir);
    // Render queues the deferred load; drain it, then render again so the decoded
    // image is resident and composes into the scene.
    runtime.render_now(&view, std::slice::from_ref(&sheet));
    let _ = runtime.poll_resource_tasks();
    let result = runtime.render_now(&view, std::slice::from_ref(&sheet));
    assert!(painted_at_y(&result, 0.0), "a.png paints unbudgeted");

    // Tightening the budget to zero bytes evicts it outright.
    let evicted = runtime.set_image_cache_budget(CacheBudget::bytes(0));
    assert_eq!(
        evicted, 1,
        "the resident image is evicted by the new budget"
    );
    assert_eq!(runtime.image_cache_statistics().entries(), 0);

    // Restoring the budget must let the very same URL load again.
    runtime.set_image_cache_budget(CacheBudget::unlimited());
    // The re-request is a fresh deferred load: render to re-queue it, drain to
    // re-decode and admit it, then render again so it composes back into the scene.
    runtime.render_now(&view, std::slice::from_ref(&sheet));
    let _ = runtime.poll_resource_tasks();
    let result = runtime.render_now(&view, std::slice::from_ref(&sheet));
    assert_eq!(
        runtime.image_cache_statistics().entries(),
        1,
        "the evicted URL was re-requested and re-decoded"
    );
    assert!(
        painted_at_y(&result, 0.0),
        "an evicted image comes back rather than silently never painting again"
    );
}

#[doc = include_str!("image_cache_budget.doc/fixed_working_set_stops_reloading.md")]
#[test]
fn fixed_working_set_stops_reloading() {
    use std::cell::RefCell;
    use std::rc::Rc;

    use obscur_kit::diagnostics::{Diagnostic, DiagnosticSink};

    #[derive(Clone, Default)]
    struct PressureReports(Rc<RefCell<usize>>);

    impl DiagnosticSink for PressureReports {
        fn emit(&self, diagnostic: &Diagnostic) {
            if diagnostic.code.as_str() == "resource.image_cache_pressure" {
                *self.0.borrow_mut() += 1;
            }
        }
    }

    let dir = temp_asset_dir();
    let mut runtime = runtime_at(&dir);
    runtime.set_image_cache_budget(CacheBudget::entries(1));
    let reports = PressureReports::default();
    runtime.set_diagnostic_sink(Box::new(reports.clone()));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        img("a.png", "a"),
        img("b.png", "b"),
    ]));
    let sheet = compile(
        ".a { position: absolute; left: 0; top: 0; }
         .b { position: absolute; left: 0; top: 80px; }",
    );
    let sheets = [sheet];

    // The first pass needs decoded intrinsic sizes, and visibility uses the
    // resulting retained boxes on the following frame.
    for _ in 0..3 {
        runtime.render_now(&view, &sheets);
        let _ = runtime.poll_resource_tasks();
    }
    let frame = runtime.render_now(&view, &sheets);
    assert!(painted_at_y(&frame, 0.0), "visible A replaced offscreen B");

    let settled = runtime.image_cache_statistics();
    for _ in 0..8 {
        runtime.render_now(&view, &sheets);
        assert_eq!(runtime.poll_resource_tasks(), 0);
    }
    assert_eq!(
        runtime.image_cache_statistics().insertions(),
        settled.insertions()
    );
    assert_eq!(
        runtime.image_cache_statistics().evictions(),
        settled.evictions()
    );

    assert_eq!(*reports.0.borrow(), 1);

    runtime.set_image_cache_budget(CacheBudget::entries(2));
    runtime.render_now(&view, &sheets);
    let _ = runtime.poll_resource_tasks();
    assert_eq!(runtime.cached_resource_counts().0, 2);
    std::fs::remove_dir_all(dir).expect("remove image fixtures");
}
