//! `VectorDecoder`: the port that turns source bytes into a vector document.

use obscur_scene::paint::SceneVector;

use crate::state::ResourceError;

/// Decodes source bytes into a [`SceneVector`] (spec §5.17).
///
/// The vector counterpart of [`ResourceDecoder`](crate::port::ResourceDecoder),
/// and a separate port for a reason: a raster decoder's product is a buffer of
/// pixels, while this one's is *geometry plus deferred paint*. Forcing both
/// through one port would mean erasing the vector document back into bytes —
/// which is exactly what §5.17 ("SVG remains vector through the scene pipeline")
/// forbids.
///
/// A host installs an implementation; the runtime uses it when an element's
/// source names a vector document. With none installed, vector sources simply do
/// not paint — the same policy an absent raster decoder has.
pub trait VectorDecoder {
    /// Parses `bytes` into a vector document.
    ///
    /// The result must keep `currentColor` unresolved
    /// ([`VectorPaint::CurrentColor`](obscur_scene::paint::VectorPaint::CurrentColor)),
    /// so one parse serves every theme — §5.17's "theme/state changes without
    /// reparsing" is a property of the decoder's output, not of its caller.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::DecodeFailed`] when `bytes` are not a valid
    /// document of this decoder's format.
    fn decode_vector(&self, bytes: &[u8]) -> Result<SceneVector, ResourceError>;
}

/// A [`VectorDecoder`] that decodes nothing — the honest default for a host that
/// installs none, so a vector source paints nothing rather than half-painting.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NullVectorDecoder;

impl VectorDecoder for NullVectorDecoder {
    fn decode_vector(&self, _bytes: &[u8]) -> Result<SceneVector, ResourceError> {
        Err(ResourceError::Unsupported(
            "no vector decoder is installed".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{NullVectorDecoder, VectorDecoder};

    #[test]
    fn the_null_decoder_refuses_rather_than_inventing_a_document() {
        assert!(NullVectorDecoder.decode_vector(b"<svg/>").is_err());
    }
}
