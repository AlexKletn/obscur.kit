//! §4.12's explicit stylesheet invalidation, reached from an application.
//!
//! §4.12 requires that "CSS parsing is cached per unique source/version and
//! invalidated **explicitly**", and the mechanism was all there: a `CssSource`
//! carries a `SourceVersion`, `StylesheetCache` is keyed by `(SourceId,
//! SourceVersion)`, and `UiRuntime::set_stylesheets` replaces the set at any time.
//! What was missing was a *caller*: `App` collected the authored sheets once before
//! the loop started, handed them over as **anonymous** sources (which the cache
//! refuses to key, by design), and never exposed the runtime again. So an
//! application could not invalidate anything, and every style edit cost a rebuild.
//!
//! These cases drive the same pair of calls `App`'s window runner makes — apply at
//! the frame boundary, run the frame — so what they prove holds for a standalone
//! window without needing one (the windowed `App` is gated on `window-winit` +
//! `render-vello`, and a test behind those features would be silently skipped by the
//! default feature set that CI runs).
#![forbid(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;

use obscur_kit::bootstrap::{CssPipeline, KitBootstrap};
use obscur_kit::control::AppControl;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, InternedSymbol, NodeId, RuntimeId};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::frame::FrameReason;
use obscur_kit::view::view::{
    CLASS, EVENT_CLICK, View, ViewEventBinding, ViewNode, ViewProperty, ViewValue,
};

/// The command the box binds, so a *handler* can be the thing that invalidates.
const THEME: InternedSymbol = InternedSymbol::named("theme");

/// The initial authored sheet.
const NARROW: &str = ".box { width: 10px; height: 40px }";

/// The replacement: the same selector, a different width.
const WIDE: &str = ".box { width: 80px; height: 40px }";

/// One clickable, sizeable box — the smallest view whose *computed* style is
/// observable through its layout box.
fn view() -> View {
    View::single(
        ViewNode::element(ComponentId::from_raw(0x4C01))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into())))
            .with_event(ViewEventBinding::new(EVENT_CLICK, THEME)),
    )
}

/// The two calls a host makes around a frame; see
/// [`AppControl::apply`](obscur_kit::control::AppControl::apply)'s host contract.
struct Host {
    runtime: UiRuntime,
    css: CssPipeline,
    control: AppControl,
}

impl Host {
    /// A runtime with the UA default sheet installed, `NARROW` as the authored set,
    /// and one frame committed.
    fn new() -> Self {
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
        KitBootstrap::new().install_default_stylesheet(&mut runtime);
        runtime.set_root_view(view);
        let mut css = CssPipeline::new();
        let control = AppControl::new();
        control.set_stylesheet(NARROW);
        control.apply(&mut runtime, &mut css);
        let mut host = Self {
            runtime,
            css,
            control,
        };
        host.frame();
        host
    }

    /// Applies whatever is queued, then runs one reactive frame, then applies again —
    /// the runner's two edges.
    fn frame(&mut self) {
        self.control.apply(&mut self.runtime, &mut self.css);
        self.runtime.request_frame(FrameReason::UiCommit);
        self.runtime
            .run_due_frame_reactive()
            .expect("a requested frame is due");
        self.control.apply(&mut self.runtime, &mut self.css);
    }

    /// The box's laid-out width — the observable end of the cascade.
    fn width(&self) -> f32 {
        let node = self.box_node();
        self.runtime
            .layout_boxes()
            .get(&node)
            .expect("the box is laid out")
            .size()
            .width
    }

    /// The retained box element (the root's only child).
    fn box_node(&self) -> NodeId {
        let root = self.runtime.tree().root().expect("a root after a frame");
        self.runtime
            .tree()
            .get(root)
            .expect("the root is retained")
            .child_nodes()[0]
    }

    /// A trusted click at the box's top-left corner.
    fn click(&mut self) {
        for kind in [PointerEventKind::Down, PointerEventKind::Up] {
            self.runtime.handle_pointer(
                PointerEvent::new(
                    PointerId::MOUSE,
                    PointerDevice::Mouse,
                    kind,
                    Point::new(2.0, 2.0),
                )
                .as_trusted(),
            );
        }
    }
}

/// **The case that was unreachable.** A replaced stylesheet changes computed style in
/// a running application: same process, same runtime, same view builder, no rebuild.
#[test]
fn a_replaced_stylesheet_changes_computed_style_without_a_rebuild() {
    let mut host = Host::new();
    assert_eq!(host.width(), 10.0, "the authored sheet decided this");

    host.control.set_stylesheet(WIDE);
    host.frame();

    assert_eq!(
        host.width(),
        80.0,
        "the replacement reached the cascade — this used to cost a recompile"
    );
}

/// The replacement replaces: a rule that was in the old set and not the new one stops
/// applying, rather than lingering because the sets were concatenated.
#[test]
fn the_authored_set_is_replaced_rather_than_appended() {
    let mut host = Host::new();
    host.control.set_stylesheet(".other { width: 5px }");
    host.frame();

    assert_eq!(
        host.width(),
        300.0,
        "nothing sizes `.box` any more, so it fills its containing block — which is \
         the UA sheet's `display: block` still doing its job"
    );
    assert!(
        host.runtime.has_default_stylesheet(),
        "the UA default sheet is not part of the authored set, so a replacement of \
         that set cannot take it away"
    );
}

/// The frame-lifecycle contract, observed: a handler queues the invalidation, nothing
/// happens while it runs, and the next frame carries it. A handler cannot install a
/// stylesheet itself — it is called from inside §2.3's propagation, with the runtime
/// mid-frame — so the queue is what makes this expressible at all.
#[test]
fn an_invalidation_queued_by_a_handler_lands_on_the_next_frame() {
    let mut host = Host::new();
    let control = host.control.clone();
    let ran = Rc::new(Cell::new(0_u32));
    {
        let counter = Rc::clone(&ran);
        host.runtime.on_command(THEME, move || {
            counter.set(counter.get() + 1);
            control.set_stylesheet(WIDE);
        });
    }

    host.click();
    assert_eq!(ran.get(), 1, "the handler ran");
    assert_eq!(
        host.width(),
        10.0,
        "and changed nothing yet: the runtime was mid-frame when it asked"
    );
    assert!(host.control.has_pending(), "the request is queued");

    host.frame();
    assert_eq!(host.width(), 80.0, "one frame later, it is the current set");
    assert!(!host.control.has_pending());
}

/// The §4.12 cache key, which the authored set never had: sheets are handed over as
/// *identified* sources, so they are cached at all, and a replacement replaces an
/// entry instead of piling up a new one. An anonymous source — what `App` used to
/// build — is deliberately never cached, because two unrelated sheets would collide
/// under one key.
#[test]
fn authored_sheets_are_cached_by_identity_and_revision() {
    let mut host = Host::new();
    assert_eq!(
        host.css.cached_sources(),
        1,
        "the authored sheet has a stable identity; an anonymous one is never cached"
    );

    host.control
        .set_stylesheets(vec![WIDE.to_owned(), ".other {}".to_owned()]);
    host.frame();
    assert_eq!(host.css.cached_sources(), 2, "one entry per position");

    host.control.set_stylesheet(NARROW);
    host.frame();
    assert_eq!(
        host.css.cached_sources(),
        2,
        "the shorter set replaced position 1's entry and left no third behind"
    );
    assert_eq!(host.width(), 10.0);
}
