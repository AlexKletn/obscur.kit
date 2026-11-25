/// Integrity metadata for a single payload file.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestFile {
    /// The relative payload path.
    pub path: Box<str>,
    /// A 64-character hexadecimal SHA-256 digest.
    pub sha256: Box<str>,
}
