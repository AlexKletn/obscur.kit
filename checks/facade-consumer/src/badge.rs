//! A component an application *authors*: the declaration, the render, the
//! registration, and the tag — with the facade as the only dependency.
//!
//! # Why this file exists
//!
//! `src/counter.rs` guards what `okui!` emits. Nothing guarded what
//! `#[derive(Contract)]` emits, and it emitted `::obscur_view::…` and
//! `::obscur_identity::…` long after the macro stopped: audit finding 2-2, the
//! residue of 1-A. The two `publish = false` crates those paths name are not in
//! this crate's extern prelude and never can be, so an application that declared
//! a component got
//!
//! ```text
//! error[E0433]: cannot find `obscur_view` in the crate root
//! ```
//!
//! from the derive alone — the mechanism §2.13 makes a component *out of*. The
//! defect survived a whole fix campaign because the fixture written to catch that
//! class of defect did not author a component; the hole in the guard was the same
//! shape as the defect. So: this crate now authors one, and compiling it is most
//! of the assertion.
//!
//! # Why the declaration is as broad as it is
//!
//! Each line of `BadgeProps` is here because it makes the derive emit a *path*
//! the others do not: a required prop and an `Option` prop reach different
//! decoders, `#[contract(default)]` a third, `#[contract(event)]` the event
//! contract, and a `slot(...)` the slot contract and its `ChildContract`. A
//! resolution failure in any one of them is a compile error in this crate, which
//! is the only place it is visible at all.
//!
//! The component is nonetheless a real one, not a syntax sampler: it renders its
//! decoded props and gives its authored children a place to land.

use obscur_kit::identity::ComponentId;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::ui_macros::{Contract, okui};
use obscur_kit::view::component::{
    Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
    ComponentRegistration, ComponentVersion, ErasedComponent, InstanceId,
};
use obscur_kit::view::view::View;

/// The text a mounted `Badge` renders for `label`, so a test can tell "the
/// component ran" from "a box appeared".
#[must_use]
pub fn badge_text(label: &str) -> String {
    format!("badge:{label}")
}

/// `Badge`'s props (spec §2.13).
///
/// `#[contract(name = "Badge")]` is what ties this Rust type to the authored tag:
/// the derive hashes that string into the component id, and the front end hashes
/// the same string when it meets `<Badge/>`. Nothing but agreement on the string
/// makes the two the same component.
/// `#[contract(style_scope = open)]` is §4.15's "explicitly open/global scopes
/// are possible", and it is here for two reasons. It is another *path* the
/// derive emits — `::obscur_kit::view::component::StyleScope` — that only a
/// crate outside this workspace can prove resolves. And it is what makes the
/// component's output part of the document for styling: scoped is the default,
/// and a scoped component's internals are reachable by nothing but a `::part()`
/// export, which is the state audit finding 2-1 described as making the tier
/// unusable for anything visual. An application that wants to style what it
/// wrote says this, once, where it declares the component.
#[derive(Contract)]
#[contract(name = "Badge")]
#[contract(slot(name = "body", cardinality = optional))]
#[contract(style_scope = open)]
pub struct BadgeProps {
    /// The badge's caption — a required prop, decoded from `label="…"`.
    pub label: String,
    /// An optional secondary line: absent is `None`, not an error.
    pub hint: Option<String>,
    /// A count that falls back to its `Default` when the author omits it.
    #[contract(default)]
    pub count: i64,
    /// The one event `Badge` exports (§2.13). It is a declaration, not a field
    /// anything reads — which is exactly why it belongs in a guard: it is the
    /// only shape that makes the derive emit an `EventContract`.
    #[contract(event)]
    pub pressed: Option<String>,
}

/// The component itself, whose render is authored with the *other* macro — so
/// this one fixture exercises both authoring surfaces the same way an
/// application does.
pub struct Badge;

impl Component for Badge {
    type Props = BadgeProps;

    fn render(&mut self, _ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
        let caption = badge_text(&props.label);
        let hint = props.hint.clone().unwrap_or_default();
        let count = props.count.to_string();
        okui! {
            <div class="badge">
                <span class="badge-label">{caption}</span>
                <span class="badge-hint">{hint}</span>
                <span class="badge-count">{count}</span>
                <div class="badge-body"><slot name="body"/></div>
            </div>
        }
    }
}

/// The factory a host registers (§2.14): the id, the contract, and how to make
/// an instance. Every one of them comes from the derived `contract()`, so the id
/// the host registers under and the id the tag lowers to cannot drift apart.
#[derive(Debug)]
pub struct BadgeFactory;

impl ComponentFactory for BadgeFactory {
    fn component_id(&self) -> ComponentId {
        BadgeProps::contract().id
    }
    fn contract(&self) -> ComponentContract {
        BadgeProps::contract()
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, BadgeProps::contract().id)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Badge)
    }
}

/// Registers `Badge` on `runtime` — the host half of §2.14's static
/// registration, written the way an application writes it.
///
/// # Panics
///
/// If the runtime refuses the registration, which for a single component with a
/// derived id means the id collided with an already-registered one.
pub fn register_badge(runtime: &mut UiRuntime) {
    runtime
        .register_component(ComponentRegistration::new(
            Box::new(BadgeFactory),
            ComponentVersion::INITIAL,
        ))
        .expect("a component an application declared registers");
}

/// A screen authoring `<Badge/>` by its tag, with content for its slot and a
/// plain `<div>` beside it.
///
/// The sibling box is not scenery. If a component tag ever fell back to the
/// generic container primitive, the host would register `Badge` under the
/// container's id, `<Badge/>` would emit that same id, the two would agree — and
/// every `<div>` on the screen would mount a badge. Agreement is not identity;
/// exactly one node here may mount.
#[must_use]
pub fn badge_screen(label: &str, count: i64) -> View {
    okui! {
        <div class="shell">
            <Badge label={label} count={count}>
                <span slot="body">"details"</span>
            </Badge>
            <div class="plain"><span>"beside"</span></div>
        </div>
    }
}
