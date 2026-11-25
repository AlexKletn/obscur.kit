use obscur_scene::paint::SceneVector;

use crate::state::ResourceError;

use super::VectorDecoder;

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
