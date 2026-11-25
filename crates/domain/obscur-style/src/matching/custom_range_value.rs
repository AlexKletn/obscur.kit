use super::StyleRangeValue;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum CustomRangeValue {
    Missing,
    Invalid,
    Numeric(StyleRangeValue),
}

impl CustomRangeValue {
    pub(super) fn numeric(&self) -> Option<&StyleRangeValue> {
        match self {
            Self::Numeric(value) => Some(value),
            Self::Missing | Self::Invalid => None,
        }
    }
}
