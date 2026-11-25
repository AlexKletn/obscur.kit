#[doc = include_str!("../../docs/api/port/layer_backend.doc/LayerFeature.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerFeature {
    #[doc = include_str!("../../docs/api/port/layer_backend.doc/Bands.md")]
    Bands,
    #[doc = include_str!("../../docs/api/port/layer_backend.doc/Reservation.md")]
    Reservation,
    #[doc = include_str!("../../docs/api/port/layer_backend.doc/FreeAnchor.md")]
    FreeAnchor,
    #[doc = include_str!("../../docs/api/port/layer_backend.doc/ExclusiveKeyboard.md")]
    ExclusiveKeyboard,
    /// Opening the surface on a nominated display rather than wherever the platform
    /// puts it.
    DisplaySelection,
    #[doc = include_str!("../../docs/api/port/layer_backend.doc/NegativeMargin.md")]
    NegativeMargin,
    /// Rendering at the display's true fractional scale rather than rounding up to an
    /// integer.
    FractionalScale,
    #[doc = include_str!("../../docs/api/port/layer_backend.doc/InputTransparency.md")]
    InputTransparency,
    /// Native popup surfaces parented to the panel.
    Popup,
    /// Keyboard focus granted on demand by the compositor.
    OnDemandKeyboard,
    /// Native text input and input-method composition for editable panel fields.
    TextInput,
}

impl LayerFeature {
    /// Every feature, so a backend's answers can be enumerated without a caller
    /// maintaining its own copy of the list — which is how a new feature comes to be
    /// silently unanswered.
    pub const ALL: [Self; 11] = [
        Self::Bands,
        Self::Reservation,
        Self::FreeAnchor,
        Self::ExclusiveKeyboard,
        Self::DisplaySelection,
        Self::NegativeMargin,
        Self::FractionalScale,
        Self::InputTransparency,
        Self::Popup,
        Self::OnDemandKeyboard,
        Self::TextInput,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_feature_roster_lists_each_feature_once() {
        let mut seen = LayerFeature::ALL.to_vec();
        seen.sort_by_key(|feature| format!("{feature:?}"));
        seen.dedup();
        assert_eq!(seen.len(), LayerFeature::ALL.len());
    }
}
