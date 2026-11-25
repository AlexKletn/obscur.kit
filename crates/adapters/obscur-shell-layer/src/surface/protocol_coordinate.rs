/// Converts an already rounded Wayland coordinate to a signed protocol field.
#[allow(clippy::cast_possible_truncation)]
pub(super) fn protocol_coordinate(rounded: f32) -> Option<i32> {
    // f32 cannot represent i32::MAX exactly, so compare in f64.
    (f64::from(i32::MIN)..=f64::from(i32::MAX))
        .contains(&f64::from(rounded))
        .then_some(rounded as i32)
}
