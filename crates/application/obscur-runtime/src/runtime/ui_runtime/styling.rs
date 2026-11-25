#![doc = include_str!("../../../docs/api/runtime/ui_runtime/styling.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_localization::locale::LanguageTag;
use obscur_style::computed::ComputedStyle;
use obscur_style::matching::Stylesheet;
use obscur_style::selector::PseudoState;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/styling.doc/set_stylesheets.md")]
    pub fn set_stylesheets(&mut self, sheets: Vec<Stylesheet>) {
        self.sheets.set_reactive(sheets);
    }

    /// Sets host-provided pointing capabilities for CSS media queries. None means the capabilities are unknown.
    pub fn set_media_input(&mut self, input: Option<obscur_style::matching::MediaInput>) {
        let previous = self.media_input();
        self.media_input_override = Some(input);
        self.invalidate_media_input(previous);
    }

    /// Restores the last media input profile reported by the platform adapter.
    pub fn clear_media_input_override(&mut self) {
        let previous = self.media_input();
        self.media_input_override = None;
        self.invalidate_media_input(previous);
    }

    /// Records the platform adapter profile. An explicit host override takes precedence.
    pub fn set_detected_media_input(&mut self, input: Option<obscur_style::matching::MediaInput>) {
        let previous = self.media_input();
        self.detected_media_input = input;
        self.invalidate_media_input(previous);
    }

    /// Returns the effective pointing capability profile, including the host override if present.
    #[must_use]
    pub fn media_input(&self) -> Option<obscur_style::matching::MediaInput> {
        self.media_input_override
            .unwrap_or(self.detected_media_input)
    }

    fn invalidate_media_input(&mut self, previous: Option<obscur_style::matching::MediaInput>) {
        if self.media_input() != previous {
            self.last_style = None;
            self.request_frame(crate::frame::FrameReason::WindowChange);
        }
    }

    /// Updates media-query preferences and schedules a restyle if any preference changed.
    pub fn set_media_preferences(&mut self, preferences: obscur_style::matching::MediaPreferences) {
        if self.media_preferences != preferences {
            self.media_preferences = preferences;
            self.last_style = None;
            self.request_frame(crate::frame::FrameReason::WindowChange);
        }
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/styling.doc/set_default_stylesheet.md")]
    pub fn set_default_stylesheet(&mut self, sheet: Stylesheet) {
        self.sheets.set_default(sheet);
    }

    /// Returns whether a UA default stylesheet has been installed via
    /// [`set_default_stylesheet`](Self::set_default_stylesheet).
    #[must_use]
    pub fn has_default_stylesheet(&self) -> bool {
        self.sheets.has_default()
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/styling.doc/set_pseudo_state.md")]
    pub fn set_pseudo_state(&mut self, node: NodeId, state: PseudoState) {
        self.pseudo.set(node, state);
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/styling.doc/clear_pseudo_states.md")]
    pub fn clear_pseudo_states(&mut self) {
        self.pseudo.clear();
    }

    /// Returns the interaction state currently recorded for `node`, or
    /// [`PseudoState::empty`] when none.
    #[must_use]
    pub fn pseudo_state(&self, node: NodeId) -> PseudoState {
        self.pseudo.get(node)
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/styling.doc/computed_style.md")]
    #[must_use]
    pub fn computed_style(&self, node: NodeId) -> Option<&ComputedStyle> {
        self.computed.get(&node)
    }

    /// Returns the application's current locale (spec §7.2); `und` until
    /// [`set_locale`](Self::set_locale) is called.
    #[must_use]
    pub fn locale(&self) -> &LanguageTag {
        &self.locale
    }
}
