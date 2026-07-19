//! **SPECIFICATION-DERIVED** component-lifecycle, registry and UI IR cases for
//! obscur.kit §2.1 (*Component model*), §2.14 (*Component registry*) and §2.15
//! (*UI IR*).
//!
//! Read this before touching anything in the file.
//!
//! Every expectation below was produced by an agent reading obscur.kit §2 with
//! **no access to this repository's source**. The cases are a statement of what
//! the implementation owes the spec, not a record of what it currently does. A
//! failure here is a defect report about the engine (or, occasionally, about the
//! spec) and must not be "corrected" by editing the expected value.
//!
//! Each test's doc comment carries the quoted normative sentence it encodes.
#![forbid(unsafe_code)]

#[path = "spec_ch2_support/mod.rs"]
mod support;

use std::cell::RefCell;
use std::rc::Rc;

use obscur_kit::capability::CapabilityId;
use obscur_kit::diagnostics::{Diagnostic, DiagnosticSink};
use obscur_kit::identity::{ComponentId, InternedSymbol};
use obscur_kit::ui_ir::codec::{UiIrDecoder, UiIrEncoder};
use obscur_kit::ui_ir::document::{UiIrDocument, UiIrNode};
use obscur_kit::ui_ir::version::{UiIrVersion, UiIrVersionRange};
use obscur_kit::view::component::{
    Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
    ComponentRegistration, ComponentRegistry, ComponentVersion, ErasedComponent, FromProps,
    InstanceId, PropsError,
};
use obscur_kit::view::contract::{CapabilityContract, CapabilityRequirement};
use obscur_kit::view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};
use support::{all_text, commit, small_runtime};

/// A sink that records every diagnostic it is handed.
#[derive(Debug)]
struct CollectingSink(Rc<RefCell<Vec<String>>>);

impl DiagnosticSink for CollectingSink {
    fn emit(&self, diagnostic: &Diagnostic) {
        self.0.borrow_mut().push(format!("{diagnostic:?}"));
    }
}

// ------------------------------------------------------------------- §2.1

/// §2.1: "- default error-boundary behavior." (with §1.7 "is contained within
/// the component subtree;" "replaces the subtree with fallback UI;")
#[test]
fn an_error_boundary_isolates_the_failing_sibling_only() {
    let mut rt = small_runtime();
    for factory in [
        Sibling::factory("S1"),
        Sibling::factory("S2"),
        Sibling::factory("S3"),
    ] {
        rt.register_component(ComponentRegistration::new(
            factory,
            ComponentVersion::INITIAL,
        ))
        .expect("the sibling registers");
    }

    let reported = Rc::new(RefCell::new(Vec::<String>::new()));
    rt.set_diagnostic_sink(Box::new(CollectingSink(Rc::clone(&reported))));

    // Frame 1: all three render.
    commit(&mut rt, &siblings(false));
    let mut first = all_text(&rt);
    first.sort();
    assert_eq!(
        first,
        ["S1:ok", "S2:ok", "S3:ok"].map(str::to_owned).to_vec(),
        "the first frame renders all three siblings"
    );

    // Frame 2: S2's view generation fails; S1 and S3 are unchanged.
    commit(&mut rt, &siblings(true));

    let text = all_text(&rt);
    assert!(
        text.contains(&"S1:ok".to_owned()) && text.contains(&"S3:ok".to_owned()),
        "S1 and S3 are still present with their prior state intact; got {text:?}"
    );
    assert!(
        text.contains(&"S2:fallback".to_owned()),
        "S2's subtree is replaced by fallback UI; got {text:?}"
    );
    assert_eq!(
        reported.borrow().len(),
        1,
        "exactly one diagnostic is reported, carrying S2's node context; got {:?}",
        reported.borrow()
    );
}

/// §2.1: "- scoped subscriptions;" and "- automatic cleanup;"
#[test]
fn a_scoped_subscription_is_unsubscribed_when_its_component_unmounts() {
    let source = obscur_kit::identity::SignalId::from_raw(0x2B01);

    let mut rt = small_runtime();
    rt.register_component(ComponentRegistration::new(
        Subscriber::factory(source),
        ComponentVersion::INITIAL,
    ))
    .expect("Subscriber registers");

    // Mount 3 components, each subscribing to the source.
    commit(&mut rt, &subscribers(3));
    assert_eq!(
        rt.notify_signal(source),
        3,
        "three mounted components subscribe to the source"
    );

    // Unmount 2 of them.
    commit(&mut rt, &subscribers(1));
    assert_eq!(
        rt.notify_signal(source),
        1,
        "subscriber count is 1 after the unmounts, and the pushed value is \
         delivered exactly once, to the surviving component"
    );

    // Unmount the last one.
    commit(&mut rt, &subscribers(0));
    assert_eq!(
        rt.notify_signal(source),
        0,
        "unmounting the last one brings the count to 0"
    );
}

// ------------------------------------------------------------------ §2.14

/// §2.14: "`ComponentRegistry` stores:" / "- stable component ID;"
#[test]
fn a_duplicate_component_id_is_refused_and_the_first_registration_wins() {
    let id = ComponentId::from_raw(0x8000_0000_0000_2B02);
    let mut registry = ComponentRegistry::new();

    registry
        .register(ComponentRegistration::new(
            Sibling::factory_with_id("A", id),
            ComponentVersion::INITIAL,
        ))
        .expect("the first registration succeeds");

    let second = registry.register(ComponentRegistration::new(
        Sibling::factory_with_id("B", id),
        ComponentVersion::INITIAL,
    ));
    assert!(
        second.is_err(),
        "the second registration is refused with a diagnostic naming the \
         conflicting ID"
    );
    let message = format!("{:?}", second.unwrap_err());
    assert!(
        message.contains(&format!("{:?}", id.raw())) || message.contains("2B02"),
        "the error names the conflicting ID; got {message}"
    );

    // A subsequent lookup resolves to A, the first, not B.
    let mut component = registry.create_component(id).expect("the id is registered");
    let mut subs = obscur_kit::view::state::SubscriptionScope::new();
    let mut needs_update = false;
    let signals = obscur_kit::view::state::SignalGenerations::new();
    let mut ctx = ComponentContext::new(
        &mut subs,
        obscur_kit::view::lifecycle::LifecycleState::Mounted,
        &mut needs_update,
        &signals,
    );
    let rendered = component
        .render_erased(&mut ctx, &[])
        .expect("the props decode");
    assert_eq!(
        text_of(&rendered),
        vec!["A:ok".to_owned()],
        "a lookup of the id resolves to A (the first), not B"
    );
}

/// §2.14: "- capability requirements;" (with §2.13 "- required capabilities;")
#[test]
fn an_ungranted_required_capability_refuses_instantiation() {
    let clipboard = CapabilityId::new("clipboard");
    let mut rt = small_runtime();
    rt.register_component(ComponentRegistration::new(
        Sibling::factory_requiring("Clip", clipboard.clone()),
        ComponentVersion::INITIAL,
    ))
    .expect("the component registers");

    let reported = Rc::new(RefCell::new(Vec::<String>::new()));
    rt.set_diagnostic_sink(Box::new(CollectingSink(Rc::clone(&reported))));

    // Instantiate it in a runtime that has not granted the capability.
    let usage = View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::element(
        Sibling::id("Clip"),
    )]));
    commit(&mut rt, &usage);

    assert_eq!(
        all_text(&rt),
        Vec::<String>::new(),
        "the component's factory is not invoked and no node is created"
    );
    let messages = reported.borrow().clone();
    assert!(
        messages
            .iter()
            .any(|m| m.contains("clipboard") || m.contains("capabilit")),
        "a diagnostic naming the component ID and the missing capability; got \
         {messages:?}"
    );
}

// ------------------------------------------------------------------ §2.15

/// §2.15: "- precise diagnostics for newer unsupported IR."
#[test]
fn a_newer_ir_version_is_refused_with_a_precise_diagnostic() {
    // The loader's supported range tops out at the current version; the
    // document declares a major version above it.
    let supported = UiIrVersionRange::new(UiIrVersion::new(1, 0), UiIrVersion::CURRENT);
    let newer = UiIrVersion::new(UiIrVersion::CURRENT.major() + 3, 0);
    assert!(
        !supported.contains(newer),
        "the fixture must actually be outside the supported range"
    );

    let document = UiIrDocument::new(UiIrNode::element(1, "div")).with_version(newer);
    let bytes = UiIrEncoder::new().encode(&document);

    let result = UiIrDecoder::new().decode(&bytes);
    let error =
        result.expect_err("loading a document declaring an unsupported newer IR version must fail");
    let message = format!("{error}");
    assert!(
        message.contains(&newer.major().to_string()),
        "the diagnostic states the document's IR version; got {message}"
    );
    assert!(
        message.contains(&supported.max().major().to_string()),
        "…and the loader's supported range; got {message}"
    );
    assert!(
        message.contains("version"),
        "…and identifies the cause as an unsupported newer version rather than \
         a generic parse/deserialize error; got {message}"
    );
}

/// §2.15: "- versioned;" and "- validated;"
#[test]
fn an_ir_document_with_no_version_field_is_rejected() {
    // A stream that is structurally valid up to the point the version would be
    // read, and then simply does not carry one.
    let full = UiIrEncoder::new().encode(&UiIrDocument::new(UiIrNode::element(1, "div")));
    // magic (4) + format version (2) = 6 bytes of header; the IR version is the
    // next four. Drop exactly those, splicing the rest back on.
    let mut headless = Vec::new();
    headless.extend_from_slice(&full[..6]);
    headless.extend_from_slice(&full[10..]);

    let error = UiIrDecoder::new()
        .decode(&headless)
        .expect_err("a document declaring no IR version must fail to load");
    let message = format!("{error}");
    assert!(
        message.contains("version"),
        "a validation diagnostic naming the missing version — it is not assumed \
         to be the current version and loaded; got {message}"
    );
}

/// §2.15: "- independent of direct Rust type names;" and "- based on stable
/// component/resource/style IDs;"
#[test]
fn ir_references_a_stable_id_and_never_a_rust_type_name() {
    // The component is implemented by a Rust type named `Button`; the IR
    // instantiates it by its stable ID.
    let document = UiIrDocument::new(
        UiIrNode::fragment(1).with_child(UiIrNode::component(2, "app.primary_button")),
    );
    let bytes = UiIrEncoder::new().encode(&document);
    let text = String::from_utf8_lossy(&bytes);

    assert!(
        text.contains("app.primary_button"),
        "the document references the stable component ID"
    );
    for type_name in ["Button", "PrimaryButtonV2"] {
        assert!(
            !text.contains(type_name),
            "the IR document's bytes show no occurrence of the Rust type name \
             `{type_name}` — a rename must not break stored documents"
        );
    }

    // The round trip resolves the same stable ID.
    let decoded = UiIrDecoder::new().decode(&bytes).expect("the IR loads");
    assert_eq!(
        decoded.root().children()[0].tag(),
        "app.primary_button",
        "the IR resolves to the component by stable ID"
    );
}

// ------------------------------------------------------------------ fixtures

/// A view holding three sibling components; `fail` makes S2's props undecodable.
fn siblings(fail: bool) -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(Sibling::id("S1")),
        ViewNode::element(Sibling::id("S2"))
            .with_prop(ViewProperty::new(FAIL, ViewValue::Bool(fail))),
        ViewNode::element(Sibling::id("S3")),
    ]))
}

/// A view holding `n` instances of the subscribing component.
fn subscribers(n: usize) -> View {
    View::from_fragment(ViewFragment::from_nodes(
        (0..n)
            .map(|_| ViewNode::element(Sibling::id("Sub")))
            .collect(),
    ))
}

/// The reserved property name a sibling reads to decide whether to fail.
const FAIL: InternedSymbol = InternedSymbol::from_raw(0x8000_2B03);

/// Props that refuse to decode when `fail` is set — the one way a component's
/// view generation can fail through the public surface.
struct SiblingProps;

impl FromProps for SiblingProps {
    fn from_props(props: &[ViewProperty]) -> Result<Self, PropsError> {
        let failing = props
            .iter()
            .any(|p| p.name() == FAIL && *p.value() == ViewValue::Bool(true));
        if failing {
            Err(PropsError::Type {
                name: FAIL,
                expected: "a renderable sibling",
            })
        } else {
            Ok(Self)
        }
    }
}

/// A sibling component that renders `"<name>:ok"`, and whose fallback UI
/// renders `"<name>:fallback"`.
struct Sibling {
    name: &'static str,
}

impl Sibling {
    /// The stable component id derived from `name`.
    fn id(name: &str) -> ComponentId {
        obscur_kit::ui_ir::name_binding::user_component_id(name)
    }

    fn factory(name: &'static str) -> Box<dyn ComponentFactory> {
        Box::new(SiblingFactory {
            name,
            id: Self::id(name),
            capability: None,
        })
    }

    fn factory_with_id(name: &'static str, id: ComponentId) -> Box<dyn ComponentFactory> {
        Box::new(SiblingFactory {
            name,
            id,
            capability: None,
        })
    }

    fn factory_requiring(
        name: &'static str,
        capability: CapabilityId,
    ) -> Box<dyn ComponentFactory> {
        Box::new(SiblingFactory {
            name,
            id: Self::id(name),
            capability: Some(capability),
        })
    }
}

impl Component for Sibling {
    type Props = SiblingProps;

    fn render(&mut self, _ctx: &mut ComponentContext<'_>, _props: &Self::Props) -> View {
        View::single(ViewNode::text(format!("{}:ok", self.name)))
    }

    fn fallback(&self) -> Option<View> {
        Some(View::single(ViewNode::text(format!(
            "{}:fallback",
            self.name
        ))))
    }
}

#[derive(Debug)]
struct SiblingFactory {
    name: &'static str,
    id: ComponentId,
    capability: Option<CapabilityId>,
}

impl ComponentFactory for SiblingFactory {
    fn component_id(&self) -> ComponentId {
        self.id
    }
    fn contract(&self) -> ComponentContract {
        let contract = ComponentContract::new(self.id);
        match &self.capability {
            Some(capability) => contract.with_capability(CapabilityContract::new(
                capability.clone(),
                CapabilityRequirement::Required,
            )),
            None => contract,
        }
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, self.id)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Sibling { name: self.name })
    }
}

/// A component that subscribes to `source` for as long as it is mounted.
struct Subscriber {
    source: obscur_kit::identity::SignalId,
}

impl Subscriber {
    fn factory(source: obscur_kit::identity::SignalId) -> Box<dyn ComponentFactory> {
        Box::new(SubscriberFactory { source })
    }
}

impl Component for Subscriber {
    type Props = SiblingProps;

    fn render(&mut self, ctx: &mut ComponentContext<'_>, _props: &Self::Props) -> View {
        ctx.track(self.source);
        View::single(ViewNode::text("sub"))
    }
}

#[derive(Debug)]
struct SubscriberFactory {
    source: obscur_kit::identity::SignalId,
}

impl ComponentFactory for SubscriberFactory {
    fn component_id(&self) -> ComponentId {
        Sibling::id("Sub")
    }
    fn contract(&self) -> ComponentContract {
        ComponentContract::new(Sibling::id("Sub"))
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, Sibling::id("Sub"))
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Subscriber {
            source: self.source,
        })
    }
}

/// Every text node's content in `v`.
fn text_of(v: &View) -> Vec<String> {
    v.root()
        .nodes()
        .iter()
        .filter_map(|n| match n {
            ViewNode::Text { content, .. } => Some(content.to_string()),
            ViewNode::Element { .. } => None,
        })
        .collect()
}
