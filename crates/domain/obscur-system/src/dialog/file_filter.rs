#[doc = include_str!("../../docs/api/dialog/dialog_request.doc/FileFilter.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFilter {
    name: Box<str>,
    extensions: Vec<Box<str>>,
}

impl FileFilter {
    /// Creates a filter from its display `name` and the `extensions` it matches
    /// (spec §6.9).
    #[must_use]
    pub fn new(
        name: impl Into<Box<str>>,
        extensions: impl IntoIterator<Item = impl Into<Box<str>>>,
    ) -> Self {
        Self {
            name: name.into(),
            extensions: extensions.into_iter().map(Into::into).collect(),
        }
    }

    /// Returns the filter's human-readable name (spec §6.9).
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the extensions this filter matches (spec §6.9).
    #[must_use]
    pub fn extensions(&self) -> &[Box<str>] {
        &self.extensions
    }
}

#[cfg(test)]
mod tests {
    use super::FileFilter;

    #[test]
    fn file_filter_stores_name_and_extensions() {
        let filter = FileFilter::new("Images", ["png", "jpg"]);
        assert_eq!(filter.name(), "Images");
        assert_eq!(filter.extensions(), &[Box::from("png"), Box::from("jpg")]);
    }
}
