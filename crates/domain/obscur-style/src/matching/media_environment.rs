#![doc = include_str!("media_environment.doc/_module.md")]

#[doc = include_str!("media_environment.doc/MediaEnvironment.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MediaEnvironment {
    viewport_width: Option<f32>,
    viewport_height: Option<f32>,
    container_width: Option<f32>,
    container_height: Option<f32>,
}

impl MediaEnvironment {
    #[doc = include_str!("media_environment.doc/unbounded.md")]
    #[must_use]
    pub const fn unbounded() -> Self {
        Self {
            viewport_width: None,
            viewport_height: None,
            container_width: None,
            container_height: None,
        }
    }

    #[doc = include_str!("media_environment.doc/viewport.md")]
    #[must_use]
    pub const fn viewport(width: f32, height: f32) -> Self {
        Self {
            viewport_width: Some(width),
            viewport_height: Some(height),
            container_width: None,
            container_height: None,
        }
    }

    #[doc = include_str!("media_environment.doc/with_container.md")]
    #[must_use]
    pub const fn with_container(mut self, width: f32, height: f32) -> Self {
        self.container_width = Some(width);
        self.container_height = Some(height);
        self
    }

    #[doc = include_str!("media_environment.doc/viewport_width.md")]
    #[must_use]
    pub const fn viewport_width(&self) -> Option<f32> {
        self.viewport_width
    }

    #[doc = include_str!("media_environment.doc/viewport_height.md")]
    #[must_use]
    pub const fn viewport_height(&self) -> Option<f32> {
        self.viewport_height
    }

    #[doc = include_str!("media_environment.doc/container_width.md")]
    #[must_use]
    pub const fn container_width(&self) -> Option<f32> {
        self.container_width
    }

    #[doc = include_str!("media_environment.doc/container_height.md")]
    #[must_use]
    pub const fn container_height(&self) -> Option<f32> {
        self.container_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unbounded_has_no_known_axes() {
        let env = MediaEnvironment::unbounded();
        assert_eq!(env.viewport_width(), None);
        assert_eq!(env.viewport_height(), None);
        assert_eq!(env.container_width(), None);
        assert_eq!(env.container_height(), None);
        assert_eq!(env, MediaEnvironment::default());
    }

    #[test]
    fn viewport_sets_width_and_height_only() {
        let env = MediaEnvironment::viewport(1000.0, 800.0);
        assert_eq!(env.viewport_width(), Some(1000.0));
        assert_eq!(env.viewport_height(), Some(800.0));
        assert_eq!(env.container_width(), None);
    }

    #[test]
    fn with_container_sets_container_axes() {
        let env = MediaEnvironment::viewport(1000.0, 800.0).with_container(200.0, 150.0);
        assert_eq!(env.container_width(), Some(200.0));
        assert_eq!(env.container_height(), Some(150.0));
        // The viewport axes are preserved.
        assert_eq!(env.viewport_width(), Some(1000.0));
    }
}
