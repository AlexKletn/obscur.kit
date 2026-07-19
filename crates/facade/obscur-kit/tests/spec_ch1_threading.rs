//! **SPECIFICATION-DERIVED** threading-model cases for obscur.kit §1.5.
//!
//! Read this before touching anything in the file.
//!
//! Every expected value below was produced by an agent reading obscur.kit §1
//! (Architecture & boundaries) with **no access to this repository's source**.
//! The cases are a statement of what the implementation owes the spec, not a
//! record of what it currently does. A failure here is a defect report about the
//! toolkit (or, occasionally, about the spec), and must not be "corrected" by
//! editing the expected value.
//!
//! Each test's doc comment carries the quoted normative sentence it encodes.
#![forbid(unsafe_code)]
#![cfg(all(feature = "resource-fs", feature = "resource-decode"))]

#[path = "spec_ch1_support/mod.rs"]
mod support;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use obscur_kit::bootstrap::KitBootstrap;
use obscur_kit::geometry::Rect;
use obscur_kit::identity::NodeId;
use obscur_kit::primitives::Image;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::scene::scene::SceneSnapshot;
use obscur_kit::view::view::{CLASS, View, ViewNode, ViewProperty, ViewValue};

use support::{
    Collected, InlineExecutor, QueuedExecutor, ThreadedExecutor, compile, el, runtime_with_id, txt,
    view,
};

/// The viewport every case in this file renders under.
const VP: f32 = 1000.0;

/// A real 2×2 opaque-red PNG, so the decode is real work.
const RED_2X2_PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 2, 0, 0, 0, 2, 8, 2, 0,
    0, 0, 253, 212, 154, 115, 0, 0, 0, 16, 73, 68, 65, 84, 120, 218, 99, 248, 207, 192, 0, 68, 12,
    16, 10, 0, 31, 238, 3, 253, 99, 94, 187, 91, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A fresh temp directory holding `a.png`, `b.png` and `c.png`.
fn temp_asset_dir() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("obscur-kit-ch1-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    for name in ["a.png", "b.png", "c.png"] {
        std::fs::write(dir.join(name), RED_2X2_PNG).expect("write png");
    }
    dir
}

/// An `Image` node classed `img`, sourced at `src`.
fn img(src: &str) -> ViewNode {
    Image::new(src)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("img".into())))
}

/// The §1.5 scenario: 3 text runs needing shaping, 2 image decodes, 1
/// stylesheet parse.
fn scenario_view() -> View {
    view(vec![el(
        10,
        "root",
        vec![
            el(11, "line", vec![txt("first run")]),
            el(12, "line", vec![txt("second run")]),
            el(13, "line", vec![txt("third run")]),
            img("a.png"),
            img("b.png"),
        ],
    )])
}

/// Every arranged node's absolute rect, in tree order — the geometry two runs
/// must agree on byte for byte.
fn geometry(rt: &UiRuntime) -> Vec<(u64, [u32; 4])> {
    let mut out: Vec<(u64, [u32; 4])> = rt
        .tree()
        .iter()
        .filter_map(|(id, _)| {
            let b = rt.layout_boxes().get(&id).copied()?;
            Some((
                id.raw(),
                [
                    b.origin().x.to_bits(),
                    b.origin().y.to_bits(),
                    b.size().width.to_bits(),
                    b.size().height.to_bits(),
                ],
            ))
        })
        .collect();
    out.sort_unstable();
    out
}

/// The scene snapshot rendered as a comparable string.
fn scene_text(snapshot: &SceneSnapshot) -> String {
    format!("{:?}", snapshot.display_list().commands())
}

/// §1.5: "Immutable scene snapshots separate UI state from renderer execution."
#[test]
fn arch_13_a_retained_scene_snapshot_never_changes_under_the_renderer() {
    let mut rt = runtime_with_id(1, VP, VP);
    let tree = view(vec![el(10, "box", vec![])]);

    let sheet1 = compile(
        ".box { position: absolute; left: 0px; top: 0px; width: 100px; height: 50px; \
         background-color: #ff0000 }",
    );
    rt.render_now(&tree, std::slice::from_ref(&sheet1));
    let s1 = rt.snapshot();
    let s1_before = scene_text(&s1);

    let sheet2 = compile(
        ".box { position: absolute; left: 200px; top: 0px; width: 300px; height: 50px; \
         background-color: #ff0000 }",
    );
    rt.render_now(&tree, std::slice::from_ref(&sheet2));
    let s2 = rt.snapshot();

    // S1 still describes the box at x=0 width 100.
    assert_eq!(
        scene_text(&s1),
        s1_before,
        "holding S1 across frame 2 must not change what S1 describes"
    );
    let s1_rects = fill_rects(&s1);
    let s2_rects = fill_rects(&s2);
    assert!(
        s1_rects
            .iter()
            .any(|r| (r.origin.x - 0.0).abs() < 1e-3 && (r.size.width - 100.0).abs() < 1e-3),
        "S1 describes the box at x=0 width 100, got {s1_rects:?}"
    );
    assert!(
        s2_rects
            .iter()
            .any(|r| (r.origin.x - 200.0).abs() < 1e-3 && (r.size.width - 300.0).abs() < 1e-3),
        "S2 describes the box at x=200 width 300, got {s2_rects:?}"
    );

    // Dropping the runtime while S1 is still held must not invalidate it.
    drop(rt);
    assert_eq!(
        scene_text(&s1),
        s1_before,
        "S1 stays readable after the runtime is dropped"
    );
}

/// The bounds of every painted path in a snapshot.
fn fill_rects(snapshot: &SceneSnapshot) -> Vec<Rect> {
    snapshot
        .display_list()
        .commands()
        .iter()
        .filter_map(|c| match c {
            PaintCommand::DrawPath { path, .. } => path.bounds(),
            _ => None,
        })
        .collect()
}

/// §1.5: "The platform adapter decides whether actual parallelism is available,
/// including WASM workers."
#[test]
fn arch_14_a_serial_adapter_and_a_parallel_one_produce_identical_output() {
    let dir = temp_asset_dir();
    let css = ".root { display: block; width: 400px } \
               .line { display: block; width: 400px; height: 20px } \
               .img { width: 40px; height: 30px }";

    let run = |executor: Arc<dyn obscur_kit::task::BlockingExecutor + Send + Sync>| {
        let mut rt = runtime_with_id(1, VP, VP);
        KitBootstrap::new().install_image_resources(&mut rt, &dir);
        rt.set_blocking_executor(executor);
        let diagnostics = Collected::install(&mut rt);
        let sheet = compile(css);
        rt.render_now(&scenario_view(), std::slice::from_ref(&sheet));
        rt.poll_resource_tasks();
        rt.render_now(&scenario_view(), std::slice::from_ref(&sheet));
        let codes: Vec<String> = diagnostics
            .all()
            .iter()
            .map(|d| d.code.as_str().to_string())
            .collect();
        (geometry(&rt), scene_text(&rt.snapshot()), codes)
    };

    let serial = run(Arc::new(InlineExecutor));
    let parallel = run(Arc::new(ThreadedExecutor));

    assert_eq!(
        serial.0, parallel.0,
        "the two adapters must produce byte-identical layout geometry"
    );
    assert_eq!(
        serial.1, parallel.1,
        "the two adapters must produce identical scene snapshots"
    );
    assert_eq!(
        serial.2, parallel.2,
        "the two adapters must produce the same diagnostics, same codes, same count"
    );
}

/// §1.5: "Cross-thread results return as typed messages and are committed at
/// deterministic synchronization points."
#[test]
fn arch_15_async_completions_commit_deterministically_at_frame_boundaries() {
    let dir = temp_asset_dir();
    let css = ".root { display: block; width: 400px } \
               .line { display: block; width: 400px; height: 20px } \
               .img { width: 40px; height: 30px }";

    // Three async loads, completed in the controlled order C, A, B.
    let once = || {
        let executor = Arc::new(QueuedExecutor::default());
        let mut rt = runtime_with_id(1, VP, VP);
        KitBootstrap::new().install_image_resources(&mut rt, &dir);
        rt.set_blocking_executor(executor.clone());
        let diagnostics = Collected::install(&mut rt);
        let sheet = compile(css);
        let tree = view(vec![el(
            10,
            "root",
            vec![img("a.png"), img("b.png"), img("c.png")],
        )]);

        rt.render_now(&tree, std::slice::from_ref(&sheet));
        assert_eq!(executor.pending(), 3, "three loads were started");

        let cached_before = rt.cached_resource_counts();
        // Complete C first.
        executor.run_at(2);
        assert_eq!(
            rt.cached_resource_counts(),
            cached_before,
            "a completion is not visible before the next frame commit"
        );
        // Then A, then B.
        executor.run_at(0);
        executor.run_at(0);

        rt.poll_resource_tasks();
        rt.render_now(&tree, std::slice::from_ref(&sheet));

        let codes: Vec<String> = diagnostics
            .all()
            .iter()
            .map(|d| d.code.as_str().to_string())
            .collect();
        (geometry(&rt), scene_text(&rt.snapshot()), codes)
    };

    let first = once();
    for run in 1..5 {
        let again = once();
        assert_eq!(
            first.0, again.0,
            "run {run}: identical retained-tree geometry"
        );
        assert_eq!(first.1, again.1, "run {run}: identical scene snapshot");
        assert_eq!(
            first.2, again.2,
            "run {run}: identical ordered diagnostics sequence"
        );
    }
}

/// Keeps `NodeId` in scope for the geometry key type.
const _: fn(NodeId) -> u64 = NodeId::raw;
