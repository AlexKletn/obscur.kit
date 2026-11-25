//! Resource-only discovery and revalidation of plugin archives.

use std::{
    fs::{self, File},
    path::Path,
};

use obscur_plugin_host::discovery::PluginDiscovery;
use obscur_plugin_model::manifest::Version;
use obscur_plugin_package::validation::{PackageError, PackageValidator};
use serde_json::json;
use sha2::{Digest, Sha256};

fn archive(path: &Path, version: &str) {
    let payload = b"not executable";
    let manifest = json!({
        "schema_version": 1, "id": "example.discovered", "name": "Discovery", "author": "Fixture",
        "version": version, "kind": "wasm",
        "compatibility": {"plugin_api": "^1", "ui_ir": "^1"},
        "entrypoint": {"path": "guest.wasm"},
        "files": [{"path": "guest.wasm", "sha256": format!("{:x}", Sha256::digest(payload))}]
    })
    .to_string();
    let mut archive = tar::Builder::new(File::create(path).unwrap());
    for (name, bytes) in [
        ("manifest.json", manifest.as_bytes()),
        ("guest.wasm", payload.as_slice()),
    ] {
        let mut header = tar::Header::new_ustar();
        header.set_mode(0o644);
        header.set_size(bytes.len() as u64);
        header.set_cksum();
        archive.append_data(&mut header, name, bytes).unwrap();
    }
    archive.finish().unwrap();
}

#[test]
fn discovers_valid_siblings_and_rejects_changes_before_loading() {
    let directory = tempfile::tempdir().unwrap();
    let good = directory.path().join("b.tar");
    let broken = directory.path().join("a.zip");
    archive(&good, "1.0.0");
    fs::write(&broken, b"bad archive").unwrap();
    fs::write(directory.path().join("ignore.txt"), b"ignore").unwrap();
    let validator = PackageValidator::new(Version::new(1, 0, 0), Version::new(1, 0, 0));
    let results = PluginDiscovery::scan(directory.path(), &validator).unwrap();
    assert!(matches!(results.as_slice(), [Err((path, _)), Ok(_)] if path == &broken));
    let found = results[1].as_ref().unwrap();
    assert_eq!(found.path(), good);
    assert_eq!(found.manifest().version, Version::new(1, 0, 0));
    assert!(found.load(&validator).is_ok());
    archive(&good, "2.0.0");
    assert!(matches!(
        found.load(&validator),
        Err(PackageError::Manifest(_))
    ));
}
