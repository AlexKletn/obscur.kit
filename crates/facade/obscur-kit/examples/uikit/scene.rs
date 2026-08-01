#![doc = include_str!("scene.doc/_module.md")]
// One entity per file (xtask forbids logic in mod.rs); the three `uikit_*`
// examples each load this module on their own, so some items — and some of the
// re-exports below — look unused inside any single binary.
#![allow(dead_code, unreachable_pub, unused_imports)]

// The scene is assembled from siblings in this same directory: the two view
// halves it composes, the classed-node vocabulary they are authored in, the
// theme table, and one file per authored stylesheet. `#[path]` is needed because
// this module is itself loaded by `#[path = "uikit/scene.rs"]` from each example
// binary, which would otherwise look for its children in `uikit/scene/`.
#[path = "classed_node.rs"]
mod classed_node;
#[path = "component_gallery.rs"]
mod component_gallery;
#[path = "dark_saas_theme.rs"]
mod dark_saas_theme;
#[path = "flat_theme.rs"]
mod flat_theme;
#[path = "fluent_theme.rs"]
mod fluent_theme;
#[path = "lara_theme.rs"]
mod lara_theme;
#[path = "material_theme.rs"]
mod material_theme;
#[path = "metro_theme.rs"]
mod metro_theme;
#[path = "minimal_theme.rs"]
mod minimal_theme;
#[path = "neutral_theme.rs"]
mod neutral_theme;
#[path = "skeuomorphic_theme.rs"]
mod skeuomorphic_theme;
#[path = "theme_catalog.rs"]
mod theme_catalog;
#[path = "theme_switcher.rs"]
mod theme_switcher;

pub use self::theme_catalog::{MATERIAL_INDEX, THEME_NAMES, themes};
pub use self::theme_switcher::KIT_TAB_CMD;

use obscur_kit::primitives::Container;
use obscur_kit::view::view::{CLASS, ID, View, ViewProperty, ViewValue};

use self::component_gallery::gallery;
use self::theme_switcher::switcher;

#[doc = include_str!("scene.doc/view.md")]
#[must_use]
pub fn view(active_theme: usize) -> View {
    let app = Container::new()
        .with_children(vec![switcher(active_theme), gallery()])
        .into_view_node()
        .with_prop(ViewProperty::new(ID, ViewValue::Text("app".into())))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("app".into())));
    View::single(app)
}
