#![doc = include_str!("../../docs/api/paint/external_buffer.doc/_module.md")]

use crate::color::AlphaMode;

use super::{BufferFormat, SceneImage};

/// A borrowed description of foreign pixels: size, row stride, layout and alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExternalBuffer<'a> {
    /// The image width in pixels.
    width: u32,
    /// The image height in pixels.
    height: u32,
    #[doc = include_str!("../../docs/api/paint/external_buffer.doc/stride.md")]
    stride: usize,
    /// How the bytes of one pixel are laid out.
    format: BufferFormat,
    /// Whether the colour channels are already multiplied by alpha.
    alpha: AlphaMode,
    /// The borrowed bytes.
    pixels: &'a [u8],
}

impl<'a> ExternalBuffer<'a> {
    #[doc = include_str!("../../docs/api/paint/external_buffer.doc/new.md")]
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

    /// The same, for a buffer with no row padding at all.
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

    /// The image width in pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// The image height in pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// The row stride in bytes.
    #[must_use]
    pub const fn stride(&self) -> usize {
        self.stride
    }

    /// The pixel layout.
    #[must_use]
    pub const fn format(&self) -> BufferFormat {
        self.format
    }

    /// Whether the buffer has no pixels — a camera that has not produced a frame yet,
    /// which is a real state and not an error.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    #[doc = include_str!("../../docs/api/paint/external_buffer.doc/to_scene_image.md")]
    #[must_use]
    pub fn to_scene_image(&self) -> SceneImage {
        let step = self.format.bytes_per_pixel();
        let row = self.width as usize * step;
        let out_row = self.width as usize * 4;
        if self.is_empty() {
            // A camera that has not produced a frame yet. Nothing to walk, and
            // a row-at-a-time walk over a zero-width row would divide the
            // buffer into chunks of nothing.
            return SceneImage::new(0, 0, Vec::new(), AlphaMode::Straight)
                .expect("an empty image holds no pixels");
        }
        // Allocated once at its final size and written into, rather than grown
        // a pixel at a time: pushing four bytes 2 million times spends most of
        // its life checking capacity, and measured at 1080p that was thirteen
        // milliseconds against the one a copy of those bytes actually costs.
        let mut out = vec![0u8; out_row * self.height as usize];
        let rows = out
            .chunks_exact_mut(out_row)
            .enumerate()
            .map(|(y, dst)| (&self.pixels[y * self.stride..y * self.stride + row], dst));
        if self.format == BufferFormat::Rgba8 {
            // Already the layout being asked for. Every row is one `memcpy`,
            // and this is what the price of a copy is SUPPOSED to be — the
            // reason a producer that can speak it should.
            for (source, dst) in rows {
                dst.copy_from_slice(source);
            }
        } else if self.format == BufferFormat::Bgra8 {
            // The most common foreign layout, and the same WIDTH as the target:
            // copy the row wholesale and swap two channels in place. A pass of
            // `memcpy` plus a pass of swaps beats reading a pixel at a time,
            // because both passes are shapes the compiler can vectorize.
            for (source, dst) in rows {
                dst.copy_from_slice(source);
                for pixel in dst.chunks_exact_mut(4) {
                    pixel.swap(0, 2);
                }
            }
        } else {
            let format = self.format;
            for (source, dst) in rows {
                for (pixel, out) in source.chunks_exact(step).zip(dst.chunks_exact_mut(4)) {
                    out.copy_from_slice(&format.to_rgba(pixel));
                }
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
        // The repacked buffer is MOVED into the image. Handing it over as an
        // `Arc<[u8]>` instead would allocate a second buffer and copy the whole
        // thing — measured at 1080p, six milliseconds a frame to publish bytes
        // that were already sitting there.
        SceneImage::new(self.width, self.height, out, alpha)
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
    fn a_zero_width_buffer_does_not_divide_its_rows_into_chunks_of_nothing() {
        // A width of zero makes an output row zero bytes wide, and walking a
        // buffer a row at a time then asks for chunks of nothing. Height alone
        // is not enough to make it empty in the eyes of that walk.
        for (w, h) in [(0, 4), (4, 0), (0, 0)] {
            let buffer =
                ExternalBuffer::packed(&[], w, h, BufferFormat::Rgba8, AlphaMode::Straight)
                    .expect("a valid empty buffer");
            assert_eq!(buffer.to_scene_image().pixels().len(), 0, "{w}x{h}");
        }
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
