//! A procedurally generated resource source (spec §5.16 generated content).

/// A resource produced by a generator rather than loaded from stored bytes (spec
/// §5.16 generated source).
///
/// A generated source names the `generator` (its stable kind, for example
/// `"solid-color"` or `"noise"`) and carries an opaque `descriptor` — a spec
/// string the generator interprets as its parameters. It is a pure description:
/// the actual bytes are produced later when a loader runs the named generator.
///
/// Because equal `(generator, descriptor)` pairs compare and hash equal, identical
/// generated requests deduplicate in the hub (spec §5.15); [`Self::dedup_key`]
/// exposes that identity as a single string for keying.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GeneratedSource {
    generator: Box<str>,
    descriptor: Box<str>,
}

impl GeneratedSource {
    /// Creates a generated source from a generator kind and its descriptor.
    #[must_use]
    pub fn new(generator: impl Into<Box<str>>, descriptor: impl Into<Box<str>>) -> Self {
        Self {
            generator: generator.into(),
            descriptor: descriptor.into(),
        }
    }

    /// Returns the generator's stable kind name (spec §5.16).
    #[must_use]
    pub fn generator(&self) -> &str {
        &self.generator
    }

    /// Returns the opaque descriptor the generator interprets (spec §5.16).
    #[must_use]
    pub fn descriptor(&self) -> &str {
        &self.descriptor
    }

    /// Returns a key combining generator and descriptor so identical generated
    /// requests deduplicate (spec §5.15).
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
