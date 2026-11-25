use super::PointerAccuracy;

/// The primary and additional pointing capabilities for media queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MediaInput {
    /// Accuracy of the primary pointer, if one is available.
    pub primary: Option<PointerAccuracy>,
    /// Whether the primary pointer can hover without activation.
    pub primary_hover: bool,
    /// Whether any secondary pointer has coarse accuracy.
    pub additional_coarse: bool,
    /// Whether any secondary pointer has fine accuracy.
    pub additional_fine: bool,
    /// Whether any secondary pointer can hover without activation.
    pub additional_hover: bool,
}

impl MediaInput {
    pub(super) fn has_pointer(self, accuracy: PointerAccuracy, any: bool) -> bool {
        self.primary == Some(accuracy)
            || (any
                && match accuracy {
                    PointerAccuracy::Coarse => self.additional_coarse,
                    PointerAccuracy::Fine => self.additional_fine,
                })
    }

    pub(super) fn can_hover(self, any: bool) -> bool {
        (self.primary.is_some() && self.primary_hover)
            || (any && (self.additional_coarse || self.additional_fine) && self.additional_hover)
    }
}
