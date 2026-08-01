#![doc = include_str!("badge.doc/_module.md")]

use obscur_kit::identity::ComponentId;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::ui_macros::{Contract, okui};
use obscur_kit::view::component::{
    Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
    ComponentRegistration, ComponentVersion, ErasedComponent, InstanceId,
};
use obscur_kit::view::view::View;

#[doc = include_str!("badge.doc/badge_text.md")]
#[must_use]
pub fn badge_text(label: &str) -> String {
    format!("badge:{label}")
}

#[doc = include_str!("badge.doc/BadgeProps.md")]
#[derive(Contract)]
#[contract(name = "Badge")]
#[contract(slot(name = "body", cardinality = optional))]
#[contract(style_scope = open)]
pub struct BadgeProps {
    #[doc = include_str!("badge.doc/label.md")]
    pub label: String,
    #[doc = include_str!("badge.doc/hint.md")]
    pub hint: Option<String>,
    #[doc = include_str!("badge.doc/count.md")]
    #[contract(default)]
    pub count: i64,
    #[doc = include_str!("badge.doc/pressed.md")]
    #[contract(event)]
    pub pressed: Option<String>,
}

#[doc = include_str!("badge.doc/Badge.md")]
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

#[doc = include_str!("badge.doc/BadgeFactory.md")]
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

#[doc = include_str!("badge.doc/register_badge.md")]
pub fn register_badge(runtime: &mut UiRuntime) {
    runtime
        .register_component(ComponentRegistration::new(
            Box::new(BadgeFactory),
            ComponentVersion::INITIAL,
        ))
        .expect("a component an application declared registers");
}

#[doc = include_str!("badge.doc/badge_screen.md")]
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
