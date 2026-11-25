//! `VectorDecoder`: the port that turns source bytes into a vector document.

use obscur_scene::paint::SceneVector;

use crate::state::ResourceError;

#[doc = include_str!("../../docs/api/port/vector_decoder.doc/VectorDecoder.md")]
pub trait VectorDecoder {
    #[doc = include_str!("../../docs/api/port/vector_decoder.doc/decode_vector.md")]
    fn decode_vector(&self, bytes: &[u8]) -> Result<SceneVector, ResourceError>;
}
