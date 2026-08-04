#![doc = include_str!("external_buffer.doc/_module.md")]

use std::sync::Arc;

use crate::color::AlphaMode;

use super::{BufferFormat, SceneImage};

#[doc = include_str!("external_buffer.doc/ExternalBuffer.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExternalBuffer<'a> {
    #[doc = include_str!("external_buffer.doc/width.md")]
    width: u32,
    #[doc = include_str!("external_buffer.doc/height.md")]
    height: u32,
    #[doc = include_str!("external_buffer.doc/stride.md")]
    stride: usize,
    #[doc = include_str!("external_buffer.doc/format.md")]
    format: BufferFormat,
    #[doc = include_str!("external_buffer.doc/alpha.md")]
    alpha: AlphaMode,
    #[doc = include_str!("external_buffer.doc/pixels.md")]
    pixels: &'a [u8],
}

impl<'a> ExternalBuffer<'a> {
    #[doc = include_str!("external_buffer.doc/new.md")]
    #[must_use]
    pub fn new(
        pixels: &'a [u8],
        width: u32,
        height: u32,
        stride: usize,
        format: BufferFormat,
        alpha: AlphaMode,
    ) -> Option<Self> {
        let row = (width as usize).checked_mul(format.bytes_per_pixel())?;
        if stride < row {
            return None;
        }
        if height > 0 {
            // The LAST row need not be padded. A producer that hands back
            // exactly `stride * height` bytes and one that trims the final
            // padding are both correct and both common, and demanding the
            // former rejects half the real buffers in the world.
            let needed = stride
                .checked_mul(height as usize - 1)
                .and_then(|full| full.checked_add(row))?;
            if pixels.len() < needed {
                return None;
            }
        }
        Some(Self {
            width,
            height,
            stride,
            format,
            alpha,
            pixels,
        })
    }

    #[doc = include_str!("external_buffer.doc/packed.md")]
    #[must_use]
    pub fn packed(
        pixels: &'a [u8],
        width: u32,
        height: u32,
        format: BufferFormat,
        alpha: AlphaMode,
    ) -> Option<Self> {
        let stride = (width as usize).checked_mul(format.bytes_per_pixel())?;
        Self::new(pixels, width, height, stride, format, alpha)
    }

    #[doc = include_str!("external_buffer.doc/width_2.md")]
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    #[doc = include_str!("external_buffer.doc/height_2.md")]
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    #[doc = include_str!("external_buffer.doc/stride_2.md")]
    #[must_use]
    pub const fn stride(&self) -> usize {
        self.stride
    }

    #[doc = include_str!("external_buffer.doc/format_2.md")]
    #[must_use]
    pub const fn format(&self) -> BufferFormat {
        self.format
    }

    #[doc = include_str!("external_buffer.doc/is_empty.md")]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    #[doc = include_str!("external_buffer.doc/to_scene_image.md")]
    #[must_use]
    pub fn to_scene_image(&self) -> SceneImage {
        let row = self.width as usize * self.format.bytes_per_pixel();
        let step = self.format.bytes_per_pixel();
        let mut out = Vec::with_capacity(self.width as usize * self.height as usize * 4);
        for y in 0..self.height as usize {
            let start = y * self.stride;
            let source = &self.pixels[start..start + row];
            for pixel in source.chunks_exact(step) {
                let mut rgba = [0u8; 4];
                self.format.write_pixel(pixel, &mut rgba);
                out.extend_from_slice(&rgba);
            }
        }
        // An opaque layout has no alpha to interpret, and the two modes agree
        // when every alpha is 255 — but saying `Straight` says the truth rather
        // than repeating whatever the caller happened to declare.
        let alpha = if self.format.carries_alpha() {
            self.alpha
        } else {
            AlphaMode::Straight
        };
        SceneImage::new(self.width, self.height, Arc::from(out), alpha)
            .expect("the buffer's own dimensions produced its own pixel count")
    }
}

#[cfg(test)]
mod tests {
    use super::{AlphaMode, BufferFormat, ExternalBuffer};

    #[test]
    fn a_tightly_packed_buffer_converts_pixel_for_pixel() {
        let pixels = [1, 2, 3, 4, 5, 6, 7, 8];
        let buffer =
            ExternalBuffer::packed(&pixels, 2, 1, BufferFormat::Rgba8, AlphaMode::Straight)
                .expect("a valid buffer");
        let image = buffer.to_scene_image();
        assert_eq!(image.width(), 2);
        assert_eq!(image.pixels(), &pixels);
    }

    #[test]
    fn row_padding_is_skipped_rather_than_drawn() {
        // The trap that makes every capture and camera buffer look sheared: a
        // stride wider than the row. Reading it as tightly packed walks the
        // padding into the picture and slides every row sideways.
        let pixels = [
            1, 1, 1, 255, 2, 2, 2, 255, 0xDE, 0xAD, // row 0 + padding
            3, 3, 3, 255, 4, 4, 4, 255, 0xDE, 0xAD, // row 1 + padding
        ];
        let buffer =
            ExternalBuffer::new(&pixels, 2, 2, 10, BufferFormat::Rgba8, AlphaMode::Straight)
                .expect("a valid buffer");
        let image = buffer.to_scene_image();
        assert_eq!(
            image.pixels(),
            &[1, 1, 1, 255, 2, 2, 2, 255, 3, 3, 3, 255, 4, 4, 4, 255],
            "no padding byte reached the picture"
        );
    }

    #[test]
    fn a_final_row_without_its_padding_is_accepted() {
        // A producer that trims the trailing padding and one that keeps it are
        // both correct and both common; demanding the padded form rejects half
        // the real buffers in the world.
        let pixels = [1, 1, 1, 255, 0xDE, 0xAD, 2, 2, 2, 255];
        assert!(
            ExternalBuffer::new(&pixels, 1, 2, 6, BufferFormat::Rgba8, AlphaMode::Straight)
                .is_some()
        );
    }

    #[test]
    fn a_buffer_too_short_for_what_it_claims_is_refused() {
        // Refused, not truncated: the alternative is reading past the end of
        // somebody else's memory.
        let pixels = [0u8; 7];
        assert!(
            ExternalBuffer::packed(&pixels, 2, 1, BufferFormat::Rgba8, AlphaMode::Straight)
                .is_none()
        );
    }

    #[test]
    fn a_stride_narrower_than_a_row_is_refused() {
        let pixels = [0u8; 64];
        assert!(
            ExternalBuffer::new(&pixels, 4, 2, 15, BufferFormat::Rgba8, AlphaMode::Straight)
                .is_none()
        );
    }

    #[test]
    fn a_bgra_camera_frame_arrives_with_its_colours_the_right_way_round() {
        // The single most common foreign layout there is. Getting it wrong does
        // not fail — it draws a blue face.
        let pixels = [30, 20, 10, 200];
        let buffer =
            ExternalBuffer::packed(&pixels, 1, 1, BufferFormat::Bgra8, AlphaMode::Premultiplied)
                .expect("a valid buffer");
        let image = buffer.to_scene_image();
        assert_eq!(image.pixels(), &[10, 20, 30, 200]);
        assert_eq!(image.alpha_mode(), AlphaMode::Premultiplied, "as declared");
    }

    #[test]
    fn an_opaque_layout_reports_straight_alpha_whatever_was_declared() {
        // There is no alpha to interpret, and saying `Straight` says the truth
        // rather than repeating what the caller happened to pass.
        let pixels = [10, 20, 30];
        let buffer =
            ExternalBuffer::packed(&pixels, 1, 1, BufferFormat::Rgb8, AlphaMode::Premultiplied)
                .expect("a valid buffer");
        assert_eq!(buffer.to_scene_image().alpha_mode(), AlphaMode::Straight);
    }

    #[test]
    fn an_empty_buffer_is_legal_and_converts_to_an_empty_image() {
        // A camera that has not produced a frame yet is a real state, and it
        // must not be an error.
        let buffer = ExternalBuffer::packed(&[], 0, 0, BufferFormat::Rgba8, AlphaMode::Straight)
            .expect("an empty buffer is a buffer");
        assert!(buffer.is_empty());
        assert_eq!(buffer.to_scene_image().pixels().len(), 0);
    }
}
