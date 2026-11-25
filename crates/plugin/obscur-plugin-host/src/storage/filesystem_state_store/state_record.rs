use std::io::{self, BufRead, Read, Write};

use obscur_plugin_model::state::StateSchema;

use super::{PluginStorage, StorageNamespace, StorageQuota};

const MAX_SCHEMA_VERSION_BYTES: usize = 1024;
const RECORD_HEADER: &[u8] = b"obscur-state-1\n";

pub(super) struct StateRecord<'a> {
    version: String,
    payload: &'a [u8],
}

impl<'a> StateRecord<'a> {
    pub(super) fn prepare(candidate: &'a PluginStorage, quota: StorageQuota) -> io::Result<Self> {
        let version = candidate.schema().version().to_string();
        let payload = candidate.bytes();
        if payload.len() > quota.max_bytes() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "candidate plugin state exceeds payload quota",
            ));
        }
        if version.len() > MAX_SCHEMA_VERSION_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "candidate plugin state schema version exceeds byte limit",
            ));
        }
        Ok(Self { version, payload })
    }

    pub(super) fn write_to(&self, mut output: impl Write) -> io::Result<()> {
        output.write_all(RECORD_HEADER)?;
        output.write_all(self.version.as_bytes())?;
        output.write_all(b"\n")?;
        output.write_all(self.payload)
    }

    pub(super) fn read(
        mut reader: impl BufRead,
        namespace: &StorageNamespace,
        quota: StorageQuota,
    ) -> io::Result<PluginStorage> {
        let mut header = [0; RECORD_HEADER.len()];
        reader.read_exact(&mut header)?;
        if header != RECORD_HEADER {
            return Err(invalid("invalid plugin state record header"));
        }
        let schema = read_schema(&mut reader)?;
        let mut bytes = Vec::new();
        reader
            .take((quota.max_bytes() as u64).saturating_add(1))
            .read_to_end(&mut bytes)?;
        if bytes.len() > quota.max_bytes() {
            return Err(invalid("stored plugin state exceeds quota"));
        }
        Ok(PluginStorage::new(namespace.clone(), schema, bytes))
    }
}

fn read_schema(mut reader: impl BufRead) -> io::Result<StateSchema> {
    let mut version = Vec::new();
    reader
        .by_ref()
        .take((MAX_SCHEMA_VERSION_BYTES + 1) as u64)
        .read_until(b'\n', &mut version)?;
    if version.pop() != Some(b'\n') {
        return Err(invalid(
            "plugin state schema version is missing or exceeds the byte limit",
        ));
    }
    let schema = std::str::from_utf8(&version)
        .map_err(invalid)?
        .parse()
        .map_err(invalid)?;
    Ok(StateSchema::new(schema))
}

fn invalid(cause: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, cause)
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_plugin_model::identity::PluginId;

    #[test]
    fn schema_version_has_its_own_byte_limit() {
        let namespace = StorageNamespace::new(PluginId::new("example").unwrap(), "install");
        let quota = StorageQuota::new(0);
        let version = format!("1.0.0+{}", "a".repeat(MAX_SCHEMA_VERSION_BYTES - 6));
        let candidate = PluginStorage::new(
            namespace.clone(),
            StateSchema::new(version.parse().unwrap()),
            Vec::new(),
        );
        let mut bytes = Vec::new();
        StateRecord::prepare(&candidate, quota)
            .unwrap()
            .write_to(&mut bytes)
            .unwrap();
        assert_eq!(
            StateRecord::read(bytes.as_slice(), &namespace, quota).unwrap(),
            candidate
        );

        let oversized_version = format!("{version}b");
        let oversized = PluginStorage::new(
            namespace.clone(),
            StateSchema::new(oversized_version.parse().unwrap()),
            Vec::new(),
        );
        assert!(matches!(
            StateRecord::prepare(&oversized, quota),
            Err(error) if error.kind() == io::ErrorKind::InvalidInput
        ));
        let mut malformed = RECORD_HEADER.to_vec();
        malformed.extend_from_slice(oversized_version.as_bytes());
        malformed.extend_from_slice(b"\npayload");
        let mut reader = malformed.as_slice();
        assert_eq!(
            StateRecord::read(&mut reader, &namespace, quota)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(reader, b"\npayload");
    }
}
