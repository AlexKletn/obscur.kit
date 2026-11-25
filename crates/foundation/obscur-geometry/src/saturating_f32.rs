/// Narrows a number to f32, saturating infinities and overflow to the nearest finite bound. NaN is preserved.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn saturating_f32(value: f64) -> f32 {
    let limit = f64::from(f32::MAX);
    value.clamp(-limit, limit) as f32
}
