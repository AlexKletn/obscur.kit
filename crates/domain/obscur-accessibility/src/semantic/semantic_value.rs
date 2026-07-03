//! The current value of an accessibility node (spec §7.1).

/// The value carried by an accessibility node (spec §7.1 value).
///
/// Different roles expose different value shapes: a text field carries
/// [`Text`](Self::Text), a toggle carries a [`Bool`](Self::Bool), and a range
/// control such as a slider or progress bar carries a bounded
/// [`Number`](Self::Number). [`None`](Self::None) is the absence of any value.
///
/// [`Number`](Self::Number) keeps its `min` and `max` alongside the value so a
/// platform adapter can report the range; [`ratio`](Self::ratio) normalises the
/// value into `[0, 1]`.
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

    /// Returns the [`Number`](Self::Number) value normalised into `[0, 1]`.
    ///
    /// The value is clamped to `[min, max]` before normalising. A degenerate
    /// range (`min == max`) yields `0.0` rather than dividing by zero. Non-numeric
    /// variants return `0.0`.
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
