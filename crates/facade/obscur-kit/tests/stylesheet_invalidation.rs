#![doc = include_str!("stylesheet_invalidation.doc/_module.md")]
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

#[doc = include_str!("stylesheet_invalidation.doc/THEME.md")]
const THEME: InternedSymbol = InternedSymbol::named("theme");

#[doc = include_str!("stylesheet_invalidation.doc/NARROW.md")]
const NARROW: &str = ".box { width: 10px; height: 40px }";

#[doc = include_str!("stylesheet_invalidation.doc/WIDE.md")]
const WIDE: &str = ".box { width: 80px; height: 40px }";

#[doc = include_str!("stylesheet_invalidation.doc/view.md")]
fn view() -> View {
    View::single(
        ViewNode::element(ComponentId::from_raw(0x4C01))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into())))
            .with_event(ViewEventBinding::new(EVENT_CLICK, THEME)),
    )
}

#[doc = include_str!("stylesheet_invalidation.doc/Host.md")]
struct Host {
    runtime: UiRuntime,
    css: CssPipeline,
    control: AppControl,
}

impl Host {
    #[doc = include_str!("stylesheet_invalidation.doc/new.md")]
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

    #[doc = include_str!("stylesheet_invalidation.doc/frame.md")]
    fn frame(&mut self) {
        self.control.apply(&mut self.runtime, &mut self.css);
        self.runtime.request_frame(FrameReason::UiCommit);
        self.runtime
            .run_due_frame_reactive()
            .expect("a requested frame is due");
        self.control.apply(&mut self.runtime, &mut self.css);
    }

    #[doc = include_str!("stylesheet_invalidation.doc/width.md")]
    fn width(&self) -> f32 {
        let node = self.box_node();
        self.runtime
            .layout_boxes()
            .get(&node)
            .expect("the box is laid out")
            .size()
            .width
    }

    #[doc = include_str!("stylesheet_invalidation.doc/box_node.md")]
    fn box_node(&self) -> NodeId {
        let root = self.runtime.tree().root().expect("a root after a frame");
        self.runtime
            .tree()
            .get(root)
            .expect("the root is retained")
            .child_nodes()[0]
    }

    #[doc = include_str!("stylesheet_invalidation.doc/click.md")]
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

#[doc = include_str!("stylesheet_invalidation.doc/a_replaced_stylesheet_changes_computed_style_without_a_rebuild.md")]
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

#[doc = include_str!("stylesheet_invalidation.doc/the_authored_set_is_replaced_rather_than_appended.md")]
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

#[doc = include_str!("stylesheet_invalidation.doc/an_invalidation_queued_by_a_handler_lands_on_the_next_frame.md")]
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

#[doc = include_str!("stylesheet_invalidation.doc/authored_sheets_are_cached_by_identity_and_revision.md")]
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
