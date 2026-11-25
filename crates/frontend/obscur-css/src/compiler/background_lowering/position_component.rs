use crate::parser::ParsedValue;

/// One classified `background-position` component.
pub(super) enum PositionComponent<T> {
    /// A horizontal edge keyword resolved to a percentage (`left` 0, `right` 100).
    Horizontal(f32),
    /// A vertical edge keyword resolved to a percentage (`top` 0, `bottom` 100).
    Vertical(f32),
    /// The `center` keyword (either axis, 50%).
    Center,
    /// An explicit `<length-percentage>`.
    Length(T),
}

/// Classifies one `background-position` component, or [`None`] when it is not a
/// valid position keyword / length.
pub(super) fn position_component<T>(
    value: &ParsedValue,
    lower: impl Fn(&ParsedValue) -> Option<T>,
) -> Option<PositionComponent<T>> {
    match value {
        ParsedValue::Keyword(keyword) => {
            if keyword.eq_ignore_ascii_case("left") {
                Some(PositionComponent::Horizontal(0.0))
            } else if keyword.eq_ignore_ascii_case("right") {
                Some(PositionComponent::Horizontal(100.0))
            } else if keyword.eq_ignore_ascii_case("top") {
                Some(PositionComponent::Vertical(0.0))
            } else if keyword.eq_ignore_ascii_case("bottom") {
                Some(PositionComponent::Vertical(100.0))
            } else if keyword.eq_ignore_ascii_case("center") {
                Some(PositionComponent::Center)
            } else {
                None
            }
        }
        _ => lower(value).map(PositionComponent::Length),
    }
}
