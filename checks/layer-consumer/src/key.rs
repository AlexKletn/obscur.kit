#![doc = include_str!("key.doc/_module.md")]

use obscur_kit::identity::ComponentId;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::ui_macros::{Contract, okui};
use obscur_kit::view::component::{
    Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
    ComponentRegistration, ComponentVersion, ErasedComponent, InstanceId,
};
use obscur_kit::view::view::View;

#[doc = include_str!("key.doc/KeyModel.md")]
#[derive(Debug, Clone)]
pub struct KeyModel {
    #[doc = include_str!("key.doc/id.md")]
    pub id: u64,
    #[doc = include_str!("key.doc/label.md")]
    pub label: String,
    #[doc = include_str!("key.doc/classes.md")]
    pub classes: String,
    #[doc = include_str!("key.doc/key_id.md")]
    pub key_id: i64,
}

#[doc = include_str!("key.doc/RowModel.md")]
#[derive(Debug, Clone)]
pub struct RowModel {
    #[doc = include_str!("key.doc/row_id.md")]
    pub id: u64,
    #[doc = include_str!("key.doc/keys.md")]
    pub keys: Vec<KeyModel>,
}

#[doc = include_str!("key.doc/KeyProps.md")]
#[derive(Contract)]
#[contract(name = "Key")]
#[contract(style_scope = open)]
pub struct KeyProps {
    #[doc = include_str!("key.doc/prop_label.md")]
    pub label: String,
    #[doc = include_str!("key.doc/prop_key_id.md")]
    #[contract(default)]
    pub key_id: i64,
    #[doc = include_str!("key.doc/prop_classes.md")]
    pub classes: Option<String>,
}

#[doc = include_str!("key.doc/Key.md")]
pub struct Key;

impl Component for Key {
    type Props = KeyProps;

    fn render(&mut self, _ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
        let caption = props.label.clone();
        let extra = props.classes.clone().unwrap_or_default();
        okui! {
            <div class="key">
                <span class="key-label">{caption}</span>
                <span class="key-classes">{extra}</span>
            </div>
        }
    }
}

#[doc = include_str!("key.doc/KeyFactory.md")]
#[derive(Debug)]
pub struct KeyFactory;

impl ComponentFactory for KeyFactory {
    fn component_id(&self) -> ComponentId {
        KeyProps::contract().id
    }
    fn contract(&self) -> ComponentContract {
        KeyProps::contract()
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, KeyProps::contract().id)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Key)
    }
}

#[doc = include_str!("key.doc/register_key.md")]
pub fn register_key(runtime: &mut UiRuntime) {
    runtime
        .register_component(ComponentRegistration::new(
            Box::new(KeyFactory),
            ComponentVersion::INITIAL,
        ))
        .expect("a component an application declared registers");
}

#[doc = include_str!("key.doc/keyboard_view.md")]
#[must_use]
pub fn keyboard_view(rows: &[RowModel]) -> View {
    okui! {
        <div class="keyboard" test-id="keyboard-root">
            <For each={rows} as={row} key={row.id}>
                <div class="row">
                    <For each={row.keys} as={entry} key={entry.id}>
                        <Key key-id={entry.key_id} label={entry.label} classes={entry.classes}/>
                    </For>
                </div>
            </For>
        </div>
    }
}

#[doc = include_str!("key.doc/sample_rows.md")]
#[must_use]
pub fn sample_rows() -> Vec<RowModel> {
    // Derived before the markup is reached, because a `key={…}` takes a path and
    // not a call — so everything a loop needs is already a field. That is the
    // shape `docs/api/09-authoring-formats.md` describes, written out.
    let row = |id: u64, labels: &[&str]| RowModel {
        id,
        keys: labels
            .iter()
            .enumerate()
            .map(|(index, label)| KeyModel {
                id: id * 100 + index as u64,
                label: (*label).to_owned(),
                classes: if label.len() > 1 { "wide" } else { "narrow" }.to_owned(),
                // A loop's `key` is a `ViewKey` (`u64`/`&str`) and a prop is a
                // `ViewValue` (`i64`, among others). The two vocabularies do not
                // meet, so an application that wants one number in both carries
                // it twice — flattened here rather than converted in the markup,
                // which cannot call anything anyway.
                key_id: (id * 100 + index as u64) as i64,
            })
            .collect(),
    };
    vec![row(1, &["q", "w", "e"]), row(2, &["shift", "a"])]
}
