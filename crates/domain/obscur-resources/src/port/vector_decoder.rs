#![doc = include_str!("vector_decoder.doc/_module.md")]

use obscur_scene::paint::SceneVector;

use crate::state::ResourceError;

#[doc = include_str!("vector_decoder.doc/VectorDecoder.md")]
pub trait VectorDecoder {
    #[doc = include_str!("vector_decoder.doc/decode_vector.md")]
    fn decode_vector(&self, bytes: &[u8]) -> Result<SceneVector, ResourceError>;
}

#[doc = include_str!("vector_decoder.doc/NullVectorDecoder.md")]
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
