//! PackagePath (skeleton stub for the spec-defined entity; not yet implemented).

/// A normalized, UTF-8, package-relative logical path.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PackagePath(Box<str>);

impl PackagePath {
    /// Rejects empty, absolute, parent-traversing, and platform-dependent paths.
    #[must_use]
    pub fn parse(path: &str) -> Option<Self> {
        if path.starts_with('/')
            || path
                .chars()
                .any(|ch| ch.is_control() || matches!(ch, '\\' | ':'))
        {
            return None;
        }
        let mut normalized = String::new();
        for part in path.split('/') {
            match part {
                "" | "." => continue,
                ".." => return None,
                part => {
                    if !normalized.is_empty() {
                        normalized.push('/');
                    }
                    normalized.push_str(part);
                }
            }
        }
        (!normalized.is_empty()).then(|| Self(normalized.into_boxed_str()))
    }

    /// The normalized slash-separated path.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
