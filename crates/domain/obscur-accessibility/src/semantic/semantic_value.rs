//! The current value of an accessibility node (spec §7.1).

#[doc = include_str!("../../docs/api/semantic/semantic_value.doc/SemanticValue.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticValue {
    /// The node has no value.
    None,
    /// Textual content, e.g. a text field's current string.
    Text(Box<str>),
    /// A boolean toggle value.
    Bool(bool),
    /// A numeric value bounded by an inclusive `[min, max]` range.
    Number {
        /// The current value.
        value: f32,
        /// The inclusive lower bound.
        min: f32,
        /// The inclusive upper bound.
        max: f32,
    },
    #[doc = include_str!("../../docs/api/semantic/semantic_value.doc/Range.md")]
    Range {
        /// The lower of the two values.
        low: f32,
        /// The upper of the two values.
        high: f32,
        /// The inclusive lower bound.
        min: f32,
        /// The inclusive upper bound.
        max: f32,
    },
}

impl SemanticValue {
    /// Creates the absent value, [`None`](Self::None).
    #[must_use]
    pub const fn none() -> Self {
        Self::None
    }

    /// Creates a textual value from anything convertible into a boxed string.
    #[must_use]
    pub fn text(text: impl Into<Box<str>>) -> Self {
        Self::Text(text.into())
    }

    /// Creates a boolean toggle value.
    #[must_use]
    pub const fn bool(value: bool) -> Self {
        Self::Bool(value)
    }

    /// Creates a bounded numeric value.
    #[must_use]
    pub const fn number(value: f32, min: f32, max: f32) -> Self {
        Self::Number { value, min, max }
    }

    /// Creates a two-ended value over an inclusive `[min, max]` scale.
    #[must_use]
    pub const fn range(low: f32, high: f32, min: f32, max: f32) -> Self {
        Self::Range {
            low,
            high,
            min,
            max,
        }
    }

    /// Returns whether this is the absent value, [`None`](Self::None).
    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Returns the textual content if this is a [`Text`](Self::Text) value.
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            _ => None,
        }
    }

    /// Returns the numeric value if this is a [`Number`](Self::Number).
    #[must_use]
    pub const fn as_number(&self) -> Option<f32> {
        match self {
            Self::Number { value, .. } => Some(*value),
            _ => None,
        }
    }

    /// Returns the two ends if this is a [`Range`](Self::Range).
    #[must_use]
    pub const fn as_range(&self) -> Option<(f32, f32)> {
        match self {
            Self::Range { low, high, .. } => Some((*low, *high)),
            _ => None,
        }
    }

    #[doc = include_str!("../../docs/api/semantic/semantic_value.doc/ratio.md")]
    #[must_use]
    pub fn ratio(&self) -> f32 {
        match self {
            Self::Number { value, min, max } => {
                if max <= min {
                    0.0
                } else {
                    ((value - min) / (max - min)).clamp(0.0, 1.0)
                }
            }
            _ => 0.0,
        }
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn none_value() {
        let v = SemanticValue::none();
        assert!(v.is_none());
        assert_eq!(v.as_text(), None);
        assert_eq!(v.as_number(), None);
    }

    #[test]
    fn text_value() {
        let v = SemanticValue::text("hello");
        assert!(!v.is_none());
        assert_eq!(v.as_text(), Some("hello"));
        assert_eq!(v.as_number(), None);
    }

    #[test]
    fn bool_value() {
        let v = SemanticValue::bool(true);
        assert_eq!(v, SemanticValue::Bool(true));
        assert_eq!(v.as_text(), None);
    }

    #[test]
    fn number_as_number() {
        let v = SemanticValue::number(3.0, 0.0, 10.0);
        assert_eq!(v.as_number(), Some(3.0));
    }

    #[test]
    fn range_carries_both_ends_and_refuses_to_pass_for_one() {
        // The whole point of the variant: a two-thumb control has two positions
        // and no single one, so `as_number` must not hand back an end. A caller
        // that got `Some(20.0)` for a 20..80 range would announce half the
        // control as if it were the whole of it.
        let v = SemanticValue::range(20.0, 80.0, 0.0, 100.0);
        assert!(!v.is_none());
        assert_eq!(v.as_range(), Some((20.0, 80.0)));
        assert_eq!(v.as_number(), None);
        assert_eq!(SemanticValue::number(5.0, 0.0, 10.0).as_range(), None);
    }

    #[test]
    fn number_ratio_normalizes() {
        assert_eq!(SemanticValue::number(5.0, 0.0, 10.0).ratio(), 0.5);
        assert_eq!(SemanticValue::number(0.0, 0.0, 10.0).ratio(), 0.0);
        assert_eq!(SemanticValue::number(10.0, 0.0, 10.0).ratio(), 1.0);
    }

    #[test]
    fn number_ratio_clamps_out_of_range() {
        assert_eq!(SemanticValue::number(-5.0, 0.0, 10.0).ratio(), 0.0);
        assert_eq!(SemanticValue::number(20.0, 0.0, 10.0).ratio(), 1.0);
    }

    #[test]
    fn number_ratio_guards_degenerate_range() {
        assert_eq!(SemanticValue::number(5.0, 5.0, 5.0).ratio(), 0.0);
    }
}
