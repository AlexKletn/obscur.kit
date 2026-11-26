use super::*;

#[test]
fn resize_starts_only_inside_the_bottom_right_target() {
    let size = LogicalSize::new(780.0, 580.0);
    assert!(TouchWindowResize::begin(1, LogicalPosition::new(760.0, 560.0), size).is_some());
    assert!(TouchWindowResize::begin(1, LogicalPosition::new(754.0, 560.0), size).is_none());
    assert!(TouchWindowResize::begin(1, LogicalPosition::new(760.0, 560.0 - 26.0), size).is_none());
    assert!(TouchWindowResize::begin(1, LogicalPosition::new(781.0, 580.0), size).is_none());
}

#[test]
fn resize_tracks_displacement_and_clamps_both_axes() {
    let size = LogicalSize::new(780.0, 580.0);
    let resize = TouchWindowResize::begin(1, LogicalPosition::new(760.0, 560.0), size).unwrap();
    let limits = WindowSizeLimits {
        minimum: Some((480.0, 360.0)),
        maximum: Some((1200.0, 900.0)),
    };
    assert_eq!(
        resize.size_at(LogicalPosition::new(810.0, 530.0), limits),
        LogicalSize::new(830.0, 550.0)
    );
    assert_eq!(
        resize.size_at(LogicalPosition::new(50.0, 1100.0), limits),
        LogicalSize::new(480.0, 900.0)
    );
}
