use super::custom_range_value::CustomRangeValue;

use std::sync::Arc;

use crate::property::{Length, LengthExpr};

use super::{ConditionComparator, MediaEnvironment, StyleQueryValue};

/// A numeric CSS value comparable within its own dimension.
#[derive(Debug, Clone, PartialEq)]
pub enum StyleRangeValue {
    /// A unitless CSS number.
    Number(f32),
    /// An absolute CSS length in logical pixels.
    Pixels(f32),
    /// A relative CSS length awaiting the selected container’s length context.
    Relative(Length),
    /// A CSS math expression using length or percentage operands.
    Math(Arc<LengthExpr>),
    /// A percentage as written in CSS, where 100 means 100%.
    Percent(f32),
}

impl StyleRangeValue {
    /// Whether values have compatible numeric dimensions, treating unitless zero as a length when needed.
    #[must_use]
    pub fn same_dimension(&self, other: &Self) -> bool {
        self.comparable_pair(other).is_some()
    }

    pub(super) fn compare(&self, comparator: ConditionComparator, reference: &Self) -> bool {
        self.comparable_pair(reference)
            .is_some_and(|(actual, expected)| comparator.compare(actual, expected))
    }

    fn comparable_pair(&self, other: &Self) -> Option<(f32, f32)> {
        use StyleRangeValue::{Number, Percent, Pixels};
        let (a, b) = match (self, other) {
            (Number(a), Number(b)) | (Pixels(a), Pixels(b)) | (Percent(a), Percent(b)) => (a, b),
            (Number(a), Pixels(b)) if *a == 0.0 => (a, b),
            (Pixels(a), Number(b)) if *b == 0.0 => (a, b),
            _ => return None,
        };
        (a.is_finite() && b.is_finite()).then_some((*a, *b))
    }

    pub(super) fn resolve(&self, env: &MediaEnvironment) -> Option<Self> {
        match self {
            Self::Relative(length) => env.resolve_style_range_length(*length).map(Self::Pixels),
            Self::Math(expr) => env.resolve_style_range_math(expr),
            numeric => Some(numeric.clone()),
        }
    }

    pub(super) fn uses_ch(&self) -> bool {
        match self {
            Self::Relative(length) => length.is_ch(),
            Self::Math(expr) => expr.uses_ch(),
            _ => false,
        }
    }

    pub(super) fn from_property(value: &crate::property::PropertyValue) -> Option<Self> {
        use crate::property::PropertyValue;
        match value {
            PropertyValue::Number(number) => Some(Self::Number(*number)),
            PropertyValue::Length(Length::Px(px)) => Some(Self::Pixels(px.0)),
            PropertyValue::Length(Length::Percent(percent)) => Some(Self::Percent(*percent)),
            PropertyValue::Length(Length::Auto) => None,
            PropertyValue::Length(length) => Some(Self::Relative(*length)),
            PropertyValue::Calc(expr) => Some(Self::Math(Arc::new(expr.clone()))),
            _ => None,
        }
    }
}

/// A literal or custom-property reference used as a numeric threshold.
pub type StyleRangeBound = StyleQueryValue<StyleRangeValue>;

impl StyleRangeBound {
    pub(super) fn resolve(&self, env: &MediaEnvironment) -> Option<StyleRangeValue> {
        self.resolve_with(|name| match env.container_style_range(name)? {
            CustomRangeValue::Missing => Some(None),
            CustomRangeValue::Invalid => None,
            CustomRangeValue::Numeric(value) => Some(Some(value)),
        })?
        .resolve(env)
    }

    pub(super) fn uses_ch(&self) -> bool {
        self.terminal_literal()
            .is_some_and(StyleRangeValue::uses_ch)
    }
}
