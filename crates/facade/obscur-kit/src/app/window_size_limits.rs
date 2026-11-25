//! Configured logical size bounds for the desktop window.

use super::AppError;

/// The optional lower and upper bounds of the desktop window content area.
#[derive(Clone, Copy, Default)]
pub(super) struct WindowSizeLimits {
    pub(super) minimum: Option<(f32, f32)>,
    pub(super) maximum: Option<(f32, f32)>,
}

impl WindowSizeLimits {
    /// Validate the configured bounds and clamp the requested initial size on each axis.
    pub(super) fn constrain_initial(
        self,
        (width, height): (f32, f32),
    ) -> Result<(f32, f32), AppError> {
        for (bound, size) in [("minimum", self.minimum), ("maximum", self.maximum)] {
            if let Some((width, height)) = size
                && (!width.is_finite() || width < 1.0 || !height.is_finite() || height < 1.0)
            {
                return Err(AppError::Window(format!(
                    "{bound} window width and height must be finite and at least one logical pixel"
                )));
            }
        }

        let (min_width, min_height) = self
            .minimum
            .unwrap_or((f32::NEG_INFINITY, f32::NEG_INFINITY));
        let (max_width, max_height) = self.maximum.unwrap_or((f32::INFINITY, f32::INFINITY));
        if min_width > max_width || min_height > max_height {
            return Err(AppError::Window(
                "minimum window size must not exceed maximum window size on either axis".into(),
            ));
        }
        Ok((
            width.clamp(min_width, max_width),
            height.clamp(min_height, max_height),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initial_size_is_clamped_on_both_axes() {
        let limits = WindowSizeLimits {
            minimum: Some((400.0, 300.0)),
            maximum: Some((900.0, 700.0)),
        };
        assert_eq!(
            limits.constrain_initial((200.0, 800.0)).unwrap(),
            (400.0, 700.0)
        );
    }

    #[test]
    fn an_unset_bound_leaves_the_axis_open() {
        let limits = WindowSizeLimits {
            minimum: Some((400.0, 300.0)),
            maximum: None,
        };
        assert_eq!(
            limits.constrain_initial((200.0, 2000.0)).unwrap(),
            (400.0, 2000.0)
        );
        assert_eq!(
            WindowSizeLimits::default()
                .constrain_initial((600.0, 500.0))
                .unwrap(),
            (600.0, 500.0)
        );
    }

    #[test]
    fn invalid_or_conflicting_bounds_report_window_errors() {
        for (minimum, maximum) in [
            (Some((f32::NAN, 300.0)), None),
            (None, Some((900.0, 0.0))),
            (Some((400.0, 300.0)), Some((900.0, 299.0))),
        ] {
            assert!(matches!(
                WindowSizeLimits { minimum, maximum }.constrain_initial((600.0, 500.0)),
                Err(AppError::Window(_))
            ));
        }
    }
}
