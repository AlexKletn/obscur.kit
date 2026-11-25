use std::{io, path::Path};

const PREFIX: &str = "generation-";
const SUFFIX_LEN: usize = 12;

pub(super) struct GenerationName<'a>(&'a str);

impl<'a> GenerationName<'a> {
    pub(super) fn temporary_directory_in(parent: &Path) -> io::Result<tempfile::TempDir> {
        let mut builder = tempfile::Builder::new();
        builder.prefix(PREFIX).rand_bytes(SUFFIX_LEN);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            builder.permissions(std::fs::Permissions::from_mode(0o700));
        }
        builder.tempdir_in(parent)
    }

    pub(super) fn parse(name: &'a str) -> io::Result<Self> {
        if name.strip_prefix(PREFIX).is_some_and(|suffix| {
            suffix.len() == SUFFIX_LEN && suffix.bytes().all(|byte| byte.is_ascii_alphanumeric())
        }) {
            Ok(Self(name))
        } else {
            Err(Self::invalid())
        }
    }

    pub(super) fn from_directory(directory: &'a Path) -> io::Result<Self> {
        let name = directory
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(Self::invalid)?;
        Self::parse(name)
    }

    pub(super) const fn as_str(&self) -> &'a str {
        self.0
    }

    fn invalid() -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid installation generation",
        )
    }
}
