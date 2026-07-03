//! Maps between domain [`SemanticAction`] and AccessKit [`Action`] (§7.1).

use accesskit::Action;
use obscur_accessibility::semantic::SemanticAction;

/// Converts between the domain [`SemanticAction`] set and AccessKit [`Action`]
/// requests in both directions (spec §7.1 actions, §2.7).
///
/// The converter is stateless. [`to_accesskit`](Self::to_accesskit) advertises
/// which actions a node supports when a tree is pushed; its inverse,
/// [`from_accesskit`](Self::from_accesskit), turns an inbound platform
/// [`accesskit::ActionRequest`]'s action back into a domain action so it can
/// enter the same command/event model as ordinary input (spec §7.1).
///
/// Not every action has a counterpart on both sides — AccessKit has no direct
/// analogue for `Select`, `Copy`, `Paste`, `Cut`, or `Dismiss`, and the domain
/// has no analogue for AccessKit's scroll-direction or `Blur` actions — so both
/// methods return [`Option`] and yield [`None`] for an unmappable action rather
/// than inventing a wrong one.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ActionConverter;

impl ActionConverter {
    /// Creates the stateless converter.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Maps a domain [`SemanticAction`] to the AccessKit [`Action`] a node
    /// should advertise, or [`None`] when AccessKit has no equivalent.
    ///
    /// [`SemanticAction::Activate`] maps to [`Action::Click`]. The clipboard
    /// actions ([`Copy`](SemanticAction::Copy), [`Paste`](SemanticAction::Paste),
    /// [`Cut`](SemanticAction::Cut)), [`Select`](SemanticAction::Select), and
    /// [`Dismiss`](SemanticAction::Dismiss) have no AccessKit action and return
    /// [`None`].
    #[must_use]
    pub const fn to_accesskit(&self, action: SemanticAction) -> Option<Action> {
        match action {
            SemanticAction::Activate => Some(Action::Click),
            SemanticAction::Focus => Some(Action::Focus),
            SemanticAction::Increment => Some(Action::Increment),
            SemanticAction::Decrement => Some(Action::Decrement),
            SemanticAction::Expand => Some(Action::Expand),
            SemanticAction::Collapse => Some(Action::Collapse),
            SemanticAction::ScrollIntoView => Some(Action::ScrollIntoView),
            SemanticAction::ShowContextMenu => Some(Action::ShowContextMenu),
            SemanticAction::SetValue => Some(Action::SetValue),
            SemanticAction::Dismiss
            | SemanticAction::Select
            | SemanticAction::Copy
            | SemanticAction::Paste
            | SemanticAction::Cut => None,
        }
    }

    /// Maps an inbound AccessKit [`Action`] back to the domain
    /// [`SemanticAction`] it should dispatch, or [`None`] when the domain has no
    /// equivalent.
    ///
    /// [`Action::Click`] maps to [`SemanticAction::Activate`]. AccessKit actions
    /// outside the shared set — the directional scroll actions, `Blur`,
    /// `CustomAction`, text-selection and tooltip actions, and so on — return
    /// [`None`].
    #[must_use]
    pub const fn from_accesskit(&self, action: Action) -> Option<SemanticAction> {
        match action {
            Action::Click => Some(SemanticAction::Activate),
            Action::Focus => Some(SemanticAction::Focus),
            Action::Increment => Some(SemanticAction::Increment),
            Action::Decrement => Some(SemanticAction::Decrement),
            Action::Expand => Some(SemanticAction::Expand),
            Action::Collapse => Some(SemanticAction::Collapse),
            Action::ScrollIntoView => Some(SemanticAction::ScrollIntoView),
            Action::ShowContextMenu => Some(SemanticAction::ShowContextMenu),
            Action::SetValue => Some(SemanticAction::SetValue),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_accesskit_maps_supported_actions() {
        let c = ActionConverter::new();
        assert_eq!(c.to_accesskit(SemanticAction::Activate), Some(Action::Click));
        assert_eq!(c.to_accesskit(SemanticAction::Focus), Some(Action::Focus));
        assert_eq!(
            c.to_accesskit(SemanticAction::Increment),
            Some(Action::Increment)
        );
        assert_eq!(
            c.to_accesskit(SemanticAction::SetValue),
            Some(Action::SetValue)
        );
        assert_eq!(
            c.to_accesskit(SemanticAction::ShowContextMenu),
            Some(Action::ShowContextMenu)
        );
    }

    #[test]
    fn to_accesskit_returns_none_for_unmappable() {
        let c = ActionConverter::new();
        assert_eq!(c.to_accesskit(SemanticAction::Copy), None);
        assert_eq!(c.to_accesskit(SemanticAction::Paste), None);
        assert_eq!(c.to_accesskit(SemanticAction::Cut), None);
        assert_eq!(c.to_accesskit(SemanticAction::Select), None);
        assert_eq!(c.to_accesskit(SemanticAction::Dismiss), None);
    }

    #[test]
    fn from_accesskit_maps_supported_actions() {
        let c = ActionConverter::new();
        assert_eq!(
            c.from_accesskit(Action::Click),
            Some(SemanticAction::Activate)
        );
        assert_eq!(c.from_accesskit(Action::Focus), Some(SemanticAction::Focus));
        assert_eq!(
            c.from_accesskit(Action::Decrement),
            Some(SemanticAction::Decrement)
        );
        assert_eq!(
            c.from_accesskit(Action::SetValue),
            Some(SemanticAction::SetValue)
        );
    }

    #[test]
    fn from_accesskit_returns_none_for_unmappable() {
        let c = ActionConverter::new();
        assert_eq!(c.from_accesskit(Action::Blur), None);
        assert_eq!(c.from_accesskit(Action::ScrollLeft), None);
        assert_eq!(c.from_accesskit(Action::CustomAction), None);
    }

    #[test]
    fn round_trips_for_shared_actions() {
        let c = ActionConverter::new();
        for action in [
            SemanticAction::Activate,
            SemanticAction::Focus,
            SemanticAction::Increment,
            SemanticAction::Decrement,
            SemanticAction::Expand,
            SemanticAction::Collapse,
            SemanticAction::ScrollIntoView,
            SemanticAction::ShowContextMenu,
            SemanticAction::SetValue,
        ] {
            let ak = c.to_accesskit(action).expect("shared action maps");
            assert_eq!(c.from_accesskit(ak), Some(action), "round trip {action:?}");
        }
    }
}
