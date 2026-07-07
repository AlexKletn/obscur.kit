//! The CSS `box-sizing` model: whether an explicit `width` / `height` sizes the
//! content box or the border box.

/// How an explicit `width` / `height` is interpreted against the box model
/// (CSS `box-sizing`, spec §2.3).
///
/// This governs only how an **explicit** size resolves into the element's
/// *content* extent (the region children are laid out within); an `Auto` size is
/// unaffected (it always fills the available content box). The distinction only
/// matters when the element has border + padding: with none, both modes are
/// identical.
///
/// * [`ContentBox`](Self::ContentBox) — the CSS default. An explicit `width` /
///   `height` is the **content** box, so the element's border box is
///   `authored + border + padding`.
/// * [`BorderBox`](Self::BorderBox) — an explicit `width` / `height` is the
///   **border** box, so the content extent is `authored − border − padding`
///   (floored at `0`) and the element's border box equals the authored size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BoxSizing {
    /// An explicit size is the content box (the CSS initial value).
    #[default]
    ContentBox,
    /// An explicit size is the border box (`box-sizing: border-box`).
    BorderBox,
}

impl BoxSizing {
    /// Maps an explicit authored size `size` into the element's **content**
    /// extent along one axis, given the axis's total border + padding `bp`.
    ///
    /// Under [`ContentBox`](Self::ContentBox) the authored value already *is* the
    /// content extent, so it passes through unchanged. Under
    /// [`BorderBox`](Self::BorderBox) the authored value is the border box, so the
    /// border + padding is subtracted and the result floored at `0` (a box
    /// smaller than its own border + padding collapses its content to zero rather
    /// than going negative).
    #[must_use]
    pub fn content_extent(self, size: f32, bp: f32) -> f32 {
        match self {
            Self::ContentBox => size,
            Self::BorderBox => (size - bp).max(0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BoxSizing;

    #[test]
    fn default_is_content_box() {
        assert_eq!(BoxSizing::default(), BoxSizing::ContentBox);
    }

    #[test]
    fn content_box_passes_size_through() {
        // The authored value is already the content extent; border + padding does
        // not shrink it.
        assert_eq!(BoxSizing::ContentBox.content_extent(52.0, 16.0), 52.0);
    }

    #[test]
    fn border_box_subtracts_border_and_padding() {
        // `height: 52; padding: 8` (bp = 16) → content extent 36.
        assert_eq!(BoxSizing::BorderBox.content_extent(52.0, 16.0), 36.0);
    }

    #[test]
    fn border_box_floors_at_zero() {
        // A box smaller than its own border + padding collapses content to zero.
        assert_eq!(BoxSizing::BorderBox.content_extent(10.0, 16.0), 0.0);
    }

    #[test]
    fn zero_border_padding_is_mode_independent() {
        for mode in [BoxSizing::ContentBox, BoxSizing::BorderBox] {
            assert_eq!(mode.content_extent(100.0, 0.0), 100.0);
        }
    }
}
