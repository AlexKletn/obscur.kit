#![doc = include_str!("generated_source.doc/_module.md")]

#[doc = include_str!("generated_source.doc/GeneratedSource.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GeneratedSource {
    generator: Box<str>,
    descriptor: Box<str>,
}

impl GeneratedSource {
    #[doc = include_str!("generated_source.doc/new.md")]
    #[must_use]
    pub fn new(generator: impl Into<Box<str>>, descriptor: impl Into<Box<str>>) -> Self {
        Self {
            generator: generator.into(),
            descriptor: descriptor.into(),
        }
    }

    #[doc = include_str!("generated_source.doc/generator.md")]
    #[must_use]
    pub fn generator(&self) -> &str {
        &self.generator
    }

    #[doc = include_str!("generated_source.doc/descriptor.md")]
    #[must_use]
    pub fn descriptor(&self) -> &str {
        &self.descriptor
    }

    #[doc = include_str!("generated_source.doc/dedup_key.md")]
    #[must_use]
    pub fn dedup_key(&self) -> String {
        format!("{}\u{1f}{}", self.generator, self.descriptor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_expose_fields() {
        let src = GeneratedSource::new("solid-color", "#ff0000");
        assert_eq!(src.generator(), "solid-color");
        assert_eq!(src.descriptor(), "#ff0000");
    }

    #[test]
    fn equal_pairs_share_dedup_key() {
        let a = GeneratedSource::new("noise", "seed=1");
        let b = GeneratedSource::new("noise", "seed=1");
        assert_eq!(a, b);
        assert_eq!(a.dedup_key(), b.dedup_key());
    }

    #[test]
    fn different_pairs_have_different_dedup_key() {
        let a = GeneratedSource::new("noise", "seed=1");
        let b = GeneratedSource::new("noise", "seed=2");
        let c = GeneratedSource::new("solid-color", "seed=1");
        assert_ne!(a.dedup_key(), b.dedup_key());
        assert_ne!(a.dedup_key(), c.dedup_key());
    }

    #[test]
    fn dedup_key_avoids_field_boundary_collision() {
        // The separator prevents ("ab", "c") from colliding with ("a", "bc").
        let a = GeneratedSource::new("ab", "c");
        let b = GeneratedSource::new("a", "bc");
        assert_ne!(a.dedup_key(), b.dedup_key());
    }
}
