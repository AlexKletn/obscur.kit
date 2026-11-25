#![doc = include_str!("../../docs/api/paint/buffer_format.doc/_module.md")]

/// The byte layout of one pixel in a foreign buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferFormat {
    /// Red, green, blue, alpha — one byte each. What the scene already speaks.
    Rgba8,
    /// Blue, green, red, alpha. The most common foreign layout there is: Windows,
    /// X11, Wayland and most of ffmpeg's packed RGB output.
    Bgra8,
    /// Red, green, blue and one byte of PADDING. Opaque — the fourth byte is not read.
    Rgbx8,
    /// Blue, green, red and one byte of PADDING. Opaque — the fourth byte is not read.
    Bgrx8,
    /// Three bytes per pixel, red first. Opaque.
    Rgb8,
    /// Three bytes per pixel, blue first. Opaque.
    Bgr8,
    /// One byte per pixel, replicated across the three colour channels. Opaque.
    Gray8,
}

impl BufferFormat {
    /// How many bytes one pixel occupies — the unit a row stride is measured in.
    #[must_use]
    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            Self::Rgba8 | Self::Bgra8 | Self::Rgbx8 | Self::Bgrx8 => 4,
            Self::Rgb8 | Self::Bgr8 => 3,
            Self::Gray8 => 1,
        }
    }

    #[doc = include_str!("../../docs/api/paint/buffer_format.doc/carries_alpha.md")]
    #[must_use]
    pub const fn carries_alpha(self) -> bool {
        matches!(self, Self::Rgba8 | Self::Bgra8)
    }

    /// Reads one pixel out as RGBA8.
    #[must_use]
    pub(crate) fn to_rgba(self, src: &[u8]) -> [u8; 4] {
        // `x` is a padding byte, not an alpha one — a producer is free to leave
        // whatever it likes there, and reading it as alpha is how a perfectly
        // opaque frame arrives invisible.
        match self {
            Self::Rgba8 => [src[0], src[1], src[2], src[3]],
            Self::Bgra8 => [src[2], src[1], src[0], src[3]],
            // A three-byte pixel and a four-byte one with a padding byte read
            // the same way: take three, force opaque.
            Self::Rgbx8 | Self::Rgb8 => [src[0], src[1], src[2], 255],
            Self::Bgrx8 | Self::Bgr8 => [src[2], src[1], src[0], 255],
            Self::Gray8 => [src[0], src[0], src[0], 255],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BufferFormat;

    #[test]
    fn each_format_states_its_own_stride_unit() {
        assert_eq!(BufferFormat::Bgra8.bytes_per_pixel(), 4);
        assert_eq!(BufferFormat::Rgb8.bytes_per_pixel(), 3);
        assert_eq!(BufferFormat::Gray8.bytes_per_pixel(), 1);
    }

    #[test]
    fn the_four_channel_orders_agree_on_the_same_pixel() {
        assert_eq!(
            BufferFormat::Rgba8.to_rgba(&[10, 20, 30, 40]),
            [10, 20, 30, 40]
        );
        assert_eq!(
            BufferFormat::Bgra8.to_rgba(&[30, 20, 10, 40]),
            [10, 20, 30, 40],
            "b and r swapped, alpha kept"
        );
    }

    #[test]
    fn a_padding_byte_is_not_read_as_alpha() {
        // The defect this guards: `x` is padding, and a producer may leave
        // anything in it. Reading it as alpha is how a perfectly opaque frame
        // arrives invisible.
        assert_eq!(
            BufferFormat::Rgbx8.to_rgba(&[10, 20, 30, 0]),
            [10, 20, 30, 255]
        );
        assert_eq!(
            BufferFormat::Bgrx8.to_rgba(&[30, 20, 10, 0]),
            [10, 20, 30, 255]
        );
        assert!(!BufferFormat::Rgbx8.carries_alpha());
    }

    #[test]
    fn a_three_byte_pixel_becomes_opaque_and_a_grey_one_replicates() {
        assert_eq!(BufferFormat::Rgb8.to_rgba(&[10, 20, 30]), [10, 20, 30, 255]);
        assert_eq!(BufferFormat::Bgr8.to_rgba(&[30, 20, 10]), [10, 20, 30, 255]);
        assert_eq!(BufferFormat::Gray8.to_rgba(&[77]), [77, 77, 77, 255]);
    }
}
