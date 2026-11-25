#![doc = include_str!("spec_ch5_resources.doc/_module.md")]
#![forbid(unsafe_code)]
#![cfg(all(feature = "resource-decode", feature = "render-cpu"))]

#[path = "spec_ch5_support/mod.rs"]
mod support;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use obscur_kit::diagnostics::{Diagnostic, DiagnosticSink};
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::primitives::Image;
use obscur_kit::resources::hub::{ResourceHub, ResourceRequest};
use obscur_kit::resources::port::{DecodedResource, LoadedBytes, ResourceDecoder, ResourceLoader};
use obscur_kit::resources::source::{MemorySource, ResourceSource};
use obscur_kit::resources::state::{ResourceError, ResourceMetadata};
use obscur_kit::resources::{ResourceHandle, ResourceKind};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewProperty, ViewValue};

use support::{at, compile, geom};

#[doc = include_str!("spec_ch5_resources.doc/VW.md")]
const VW: f32 = 200.0;

#[doc = include_str!("spec_ch5_resources.doc/Recorder.md")]
struct Recorder {
    seen: Rc<RefCell<Vec<Diagnostic>>>,
}

impl DiagnosticSink for Recorder {
    fn emit(&self, diagnostic: &Diagnostic) {
        self.seen.borrow_mut().push(diagnostic.clone());
    }
}

#[doc = include_str!("spec_ch5_resources.doc/CountingLoader.md")]
struct CountingLoader {
    calls: Arc<AtomicUsize>,
    bytes: Arc<[u8]>,
}

impl ResourceLoader for CountingLoader {
    fn can_load(&self, _source: &ResourceSource) -> bool {
        true
    }

    fn load(&self, _source: &ResourceSource) -> Result<LoadedBytes, ResourceError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(LoadedBytes::new(
            Arc::clone(&self.bytes),
            ResourceMetadata::new(),
        ))
    }
}

#[doc = include_str!("spec_ch5_resources.doc/NeverLoader.md")]
struct NeverLoader {
    calls: Arc<AtomicUsize>,
}

impl ResourceLoader for NeverLoader {
    fn can_load(&self, _source: &ResourceSource) -> bool {
        true
    }

    fn load(&self, _source: &ResourceSource) -> Result<LoadedBytes, ResourceError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Err(ResourceError::Cancelled)
    }
}

#[doc = include_str!("spec_ch5_resources.doc/GarbageDecoder.md")]
struct GarbageDecoder;

impl ResourceDecoder for GarbageDecoder {
    fn supports(&self, _kind: ResourceKind) -> bool {
        true
    }

    fn decode(&self, _kind: ResourceKind, _bytes: &[u8]) -> Result<DecodedResource, ResourceError> {
        Err(ResourceError::DecodeFailed("not an image".into()))
    }
}

#[doc = include_str!("spec_ch5_resources.doc/rs_01_a_stale_decode_job_is_rejected.md")]
#[test]
fn rs_01_a_stale_decode_job_is_rejected() {
    let mut hub = ResourceHub::new();
    let handle: ResourceHandle = hub.request(ResourceRequest::new(
        ResourceSource::memory(MemorySource::new(vec![1_u8, 2, 3])),
        ResourceKind::Image,
    ));

    // Version 1's job starts but does not complete.
    let v1 = hub.begin_load(handle).expect("v1 begins");

    // A hot reload supersedes it; version 2's job completes first.
    let v2 = hub.reload(handle).expect("v2 supersedes v1");
    assert!(v2.is_newer_than(v1), "the reload advanced the version");
    let v2_metadata = ResourceMetadata::new().with_byte_size(2);
    assert!(
        hub.complete(handle, v2, v2_metadata.clone()),
        "v2's completion is accepted"
    );
    assert_eq!(
        hub.state(handle).and_then(|s| s.version()),
        Some(v2),
        "the handle is at version 2"
    );

    // Now version 1's job finishes — late. It must be discarded.
    let v1_metadata = ResourceMetadata::new().with_byte_size(1);
    assert!(
        !hub.complete(handle, v1, v1_metadata),
        "the stale v1 completion is REJECTED"
    );
    assert_eq!(
        hub.state(handle).and_then(|s| s.version()),
        Some(v2),
        "the handle's version stays 2"
    );
    assert_eq!(
        hub.state(handle)
            .and_then(|s| s.metadata())
            .and_then(obscur_kit::resources::state::ResourceMetadata::byte_size),
        Some(2),
        "v2's result survives; the v1 result did not overwrite it"
    );
}

#[doc = include_str!("spec_ch5_resources.doc/rs_02_no_loader_means_no_implicit_network_access.md")]
#[test]
fn rs_02_no_loader_means_no_implicit_network_access() {
    let seen: Rc<RefCell<Vec<Diagnostic>>> = Rc::default();
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VW, VW));
    runtime.set_diagnostic_sink(Box::new(Recorder {
        seen: Rc::clone(&seen),
    }));
    // NO loader and NO decoder are installed.

    let sheet = compile(
        ".a{position:absolute;left:0;top:0;width:50px;height:50px;\
            background-image:url(\"https://example.com/a.png\")}\
         .b{position:absolute;left:60px;top:0;width:50px;height:50px}",
    );
    let remote = Image::new("https://example.com/b.png")
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("b".into())));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        support::el(10, "a", vec![]),
        remote,
    ]));

    // Render until both handles settle. No panic at any point.
    for _ in 0..5 {
        let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
    }

    assert_eq!(
        seen.borrow().len(),
        2,
        "two diagnostics, one per unreachable URL, naming their sources; got {:?}",
        seen.borrow()
            .iter()
            .map(|d| (d.code.as_str(), d.message.clone()))
            .collect::<Vec<_>>()
    );

    // Both nodes still lay out and the frame still presents.
    let (_, _, w, h) = geom(&runtime, at(&runtime, &[0]));
    assert_eq!((w, h), (50.0, 50.0), "the url() node still lays out");

    // With a loader registered the kit routes through it — and only through it.
    let calls = Arc::new(AtomicUsize::new(0));
    let mut wired = UiRuntime::new(RuntimeId::from_raw(2), Size::new(VW, VW));
    wired.set_resource_providers(
        Arc::new(CountingLoader {
            calls: Arc::clone(&calls),
            bytes: Arc::from(vec![0_u8; 4]),
        }),
        Arc::new(GarbageDecoder),
    );
    for _ in 0..5 {
        let _ = wired.render_now(&view, std::slice::from_ref(&sheet));
    }
    // With no blocking executor the load is deferred out of the frame phase
    // (§3.4/§5.15); draining it on poll is what runs the loader.
    let _ = wired.poll_resource_tasks();
    assert_eq!(
        calls.load(Ordering::Relaxed),
        2,
        "the host loader is invoked exactly twice — once per URL — however many \
         frames run"
    );
}

#[doc = include_str!("spec_ch5_resources.doc/rs_03_a_failed_load_is_a_state_not_a_panic_and_not_a_retry_loop.md")]
#[test]
fn rs_03_a_failed_load_is_a_state_not_a_panic_and_not_a_retry_loop() {
    let seen: Rc<RefCell<Vec<Diagnostic>>> = Rc::default();
    let calls = Arc::new(AtomicUsize::new(0));

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VW, VW));
    runtime.set_diagnostic_sink(Box::new(Recorder {
        seen: Rc::clone(&seen),
    }));
    runtime.set_resource_providers(
        Arc::new(CountingLoader {
            calls: Arc::clone(&calls),
            // 12 bytes of non-image garbage.
            bytes: Arc::from(b"not an image".to_vec()),
        }),
        Arc::new(GarbageDecoder),
    );

    let sheet = compile(".pic{position:absolute;left:0;top:0;width:40px;height:30px}");
    let node = Image::new("garbage.png")
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("pic".into())));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![node]));

    for _ in 0..5 {
        let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
    }
    // With no blocking executor the load is deferred out of the frame phase
    // (§3.4/§5.15). The failed URL was queued ONCE (marked pending at enqueue
    // time), so the five no-poll frames re-queue nothing and a single drain runs
    // the loader exactly once.
    let _ = runtime.poll_resource_tasks();

    assert_eq!(
        calls.load(Ordering::Relaxed),
        1,
        "the loader is invoked exactly once over five frames — no retry loop"
    );
    assert_eq!(
        seen.borrow().len(),
        1,
        "exactly one diagnostic, with a stable code and the resource context; \
         got {:?}",
        seen.borrow()
            .iter()
            .map(|d| (d.code.as_str(), d.context.resource))
            .collect::<Vec<_>>()
    );
    assert!(
        seen.borrow()[0].context.resource.is_some(),
        "the diagnostic carries the resource context (§1.6)"
    );

    // The node still participates in layout at its specified size.
    let (_, _, w, h) = geom(&runtime, at(&runtime, &[0]));
    assert_eq!(
        (w, h),
        (40.0, 30.0),
        "the failed image node still lays out at its specified size"
    );
}

#[doc = include_str!("spec_ch5_resources.doc/st_01_the_first_frame_does_not_wait_on_resources.md")]
#[test]
fn st_01_the_first_frame_does_not_wait_on_resources() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VW, VW));
    runtime.set_resource_providers(
        Arc::new(NeverLoader {
            calls: Arc::clone(&calls),
        }),
        Arc::new(GarbageDecoder),
    );

    let sheet = compile(
        ".pic{position:absolute;left:0;top:0;width:40px;height:30px}\
         .label{position:absolute;left:0;top:40px;font-size:16px}",
    );
    let picture = Image::new("pending.png")
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("pic".into())));
    let label = support::el(11, "label", vec![support::txt("Hello")]);
    let view = View::from_fragment(ViewFragment::from_nodes(vec![picture, label]));

    // The very first frame is produced, with the resource still unresolved.
    let result = runtime.render_now(&view, std::slice::from_ref(&sheet));
    let list = result.snapshot().display_list().commands().to_vec();

    // The image node lays out at its placeholder box.
    let (_, _, w, h) = geom(&runtime, at(&runtime, &[0]));
    assert_eq!(
        (w, h),
        (40.0, 30.0),
        "the pending image node still occupies its box on the FIRST frame"
    );

    // The text paints with a fallback face and has non-zero advance — it is not
    // invisible while a webfont is pending.
    let advance: f32 = list
        .iter()
        .filter_map(|c| match c {
            PaintCommand::DrawText { run, .. } => Some(run.advance()),
            _ => None,
        })
        .sum();
    assert!(
        advance > 0.0,
        "the label paints with a fallback face and non-zero advance on the \
         first frame; the display list is {:?}",
        list.iter().map(support::describe).collect::<Vec<_>>()
    );
}

#[doc = include_str!("spec_ch5_resources.doc/zero_extent_image_reports_decode_failure.md")]
#[test]
fn zero_extent_image_reports_decode_failure() {
    struct EmptyDecoder;

    impl ResourceDecoder for EmptyDecoder {
        fn supports(&self, kind: ResourceKind) -> bool {
            kind == ResourceKind::Image
        }

        fn decode(
            &self,
            _kind: ResourceKind,
            _bytes: &[u8],
        ) -> Result<DecodedResource, ResourceError> {
            Ok(DecodedResource::new(
                Arc::from([]),
                ResourceMetadata::new().with_intrinsic_size(Size::new(0.0, 0.0)),
            ))
        }
    }

    let seen = Rc::new(RefCell::new(Vec::new()));
    let calls = Arc::new(AtomicUsize::new(0));
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VW, VW));
    runtime.set_diagnostic_sink(Box::new(Recorder {
        seen: Rc::clone(&seen),
    }));
    runtime.set_resource_providers(
        Arc::new(CountingLoader {
            calls: Arc::clone(&calls),
            bytes: Arc::from([]),
        }),
        Arc::new(EmptyDecoder),
    );
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        Image::new("empty.png").into_view_node(),
    ]));
    let _ = runtime.render_now(&view, &[]);
    assert_eq!(runtime.poll_resource_tasks(), 0);
    let result = runtime.render_now(&view, &[]);
    assert_eq!(runtime.poll_resource_tasks(), 0);
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert_eq!(runtime.cached_resource_counts(), (0, 0));
    assert!(
        !result
            .snapshot()
            .display_list()
            .commands()
            .iter()
            .any(|command| matches!(command, PaintCommand::DrawImage { .. }))
    );

    let diagnostics = seen.borrow();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code.as_str(), "resource.load_failed");
    assert!(diagnostics[0].message.contains("empty.png"));
    assert!(
        diagnostics[0]
            .message
            .contains("decoded image has zero extent")
    );
}

#[doc = include_str!("spec_ch5_resources.doc/panicking_decoder_reports_load_failure.md")]
#[test]
fn panicking_decoder_reports_load_failure() {
    struct PanickingDecoder;

    impl ResourceDecoder for PanickingDecoder {
        fn supports(&self, kind: ResourceKind) -> bool {
            kind == ResourceKind::Image
        }

        fn decode(
            &self,
            _kind: ResourceKind,
            _bytes: &[u8],
        ) -> Result<DecodedResource, ResourceError> {
            panic!("test decoder panic");
        }
    }

    for threaded in [false, cfg!(feature = "task-smol")] {
        let seen = Rc::new(RefCell::new(Vec::new()));
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VW, VW));
        runtime.set_diagnostic_sink(Box::new(Recorder {
            seen: Rc::clone(&seen),
        }));
        #[cfg(feature = "task-smol")]
        if threaded {
            obscur_kit::bootstrap::KitBootstrap::new().install_blocking_executor(&mut runtime);
        }
        runtime.set_resource_providers(
            Arc::new(CountingLoader {
                calls: Arc::new(AtomicUsize::new(0)),
                bytes: Arc::from([]),
            }),
            Arc::new(PanickingDecoder),
        );
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            Image::new("panic.png").into_view_node(),
        ]));
        let _ = runtime.render_now(&view, &[]);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while seen.borrow().is_empty() {
            assert!(
                std::time::Instant::now() < deadline,
                "missing failure, threaded={threaded}"
            );
            let _ = runtime.poll_resource_tasks();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        let _ = runtime.render_now(&view, &[]);
        let _ = runtime.poll_resource_tasks();
        let diagnostics = seen.borrow();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code.as_str(), "resource.load_failed");
        assert!(diagnostics[0].message.contains("panic.png"));
        assert!(
            diagnostics[0]
                .message
                .contains("resource loader or decoder panicked")
        );
        assert_eq!(runtime.cached_resource_counts(), (0, 0));
    }
}
