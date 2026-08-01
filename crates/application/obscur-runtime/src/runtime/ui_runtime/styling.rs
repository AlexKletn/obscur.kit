#![doc = include_str!("styling.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_localization::locale::LanguageTag;
use obscur_style::computed::ComputedStyle;
use obscur_style::matching::Stylesheet;
use obscur_style::selector::PseudoState;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("styling.doc/set_stylesheets.md")]
    pub fn set_stylesheets(&mut self, sheets: Vec<Stylesheet>) {
        self.sheets.set_reactive(sheets);
    }

    #[doc = include_str!("styling.doc/set_default_stylesheet.md")]
    pub fn set_default_stylesheet(&mut self, sheet: Stylesheet) {
        self.sheets.set_default(sheet);
    }

    #[doc = include_str!("styling.doc/has_default_stylesheet.md")]
    #[must_use]
    pub fn has_default_stylesheet(&self) -> bool {
        self.sheets.has_default()
    }

    #[doc = include_str!("styling.doc/set_pseudo_state.md")]
    pub fn set_pseudo_state(&mut self, node: NodeId, state: PseudoState) {
        self.pseudo.set(node, state);
    }

    #[doc = include_str!("styling.doc/clear_pseudo_states.md")]
    pub fn clear_pseudo_states(&mut self) {
        self.pseudo.clear();
    }

    #[doc = include_str!("styling.doc/pseudo_state.md")]
    #[must_use]
    pub fn pseudo_state(&self, node: NodeId) -> PseudoState {
        self.pseudo.get(node)
    }

    #[doc = include_str!("styling.doc/computed_style.md")]
    #[must_use]
    pub fn computed_style(&self, node: NodeId) -> Option<&ComputedStyle> {
        self.computed.get(&node)
    }

    #[doc = include_str!("styling.doc/locale.md")]
    #[must_use]
    pub fn locale(&self) -> &LanguageTag {
        &self.locale
    }
}
