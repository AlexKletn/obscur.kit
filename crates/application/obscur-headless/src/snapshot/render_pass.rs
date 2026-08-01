#![doc = include_str!("render_pass.doc/_module.md")]

#[doc = include_str!("render_pass.doc/RenderPass.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPass {
    #[doc = include_str!("render_pass.doc/label.md")]
    label: Box<str>,
    #[doc = include_str!("render_pass.doc/draw_count.md")]
    draw_count: usize,
    #[doc = include_str!("render_pass.doc/command_count.md")]
    command_count: usize,
}

impl RenderPass {
    #[doc = include_str!("render_pass.doc/new.md")]
    #[must_use]
    pub fn new(label: impl Into<Box<str>>, draw_count: usize, command_count: usize) -> Self {
        Self {
            label: label.into(),
            draw_count,
            command_count,
        }
    }

    #[doc = include_str!("render_pass.doc/label_2.md")]
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    #[doc = include_str!("render_pass.doc/draw_count_2.md")]
    #[must_use]
    pub fn draw_count(&self) -> usize {
        self.draw_count
    }

    #[doc = include_str!("render_pass.doc/command_count_2.md")]
    #[must_use]
    pub fn command_count(&self) -> usize {
        self.command_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_read_fields() {
        let pass = RenderPass::new("scene", 3, 7);
        assert_eq!(pass.label(), "scene");
        assert_eq!(pass.draw_count(), 3);
        assert_eq!(pass.command_count(), 7);
    }

    #[test]
    fn equal_passes_compare_equal() {
        assert_eq!(
            RenderPass::new("scene", 1, 2),
            RenderPass::new("scene", 1, 2)
        );
        assert_ne!(
            RenderPass::new("scene", 1, 2),
            RenderPass::new("scene", 1, 3)
        );
    }
}
