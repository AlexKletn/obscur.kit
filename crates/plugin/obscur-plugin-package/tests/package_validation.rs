#![doc = include_str!("package_validation.doc/_module.md")]

use std::io::Write;

use flate2::{Compression, write::GzEncoder};
use obscur_plugin_model::identity::PluginId;
use obscur_plugin_package::{
    archive::{TarPackageReader, ZipPackageReader},
    extraction::ExtractionPlan,
    package::{PackageFormat, PluginPackage},
    path::PackagePath,
    validation::{PackageError, PackageLimits, PackageValidator},
};
use semver::Version;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tar::{Builder, EntryType, Header};

const ENTRYPOINT: &str = "bin/panel.wasm";
const PAYLOAD: &[u8] = b"package payload";

fn manifest() -> Value {
    json!({
        "schema_version": 1,
        "id": "example.panel",
        "name": "Example panel",
        "version": "1.3.0",
        "author": "Example",
        "kind": "wasm",
        "compatibility": { "plugin_api": "^2", "ui_ir": "1" },
        "entrypoint": { "path": "./bin/panel.wasm" },
        "files": [{ "path": ENTRYPOINT, "sha256": format!("{:x}", Sha256::digest(PAYLOAD)) }]
    })
}

fn archive(entries: &[(&str, EntryType, &[u8])]) -> Vec<u8> {
    let mut archive = Builder::new(Vec::new());
    for (name, kind, bytes) in entries {
        let mut header = Header::new_ustar();
        header.set_entry_type(*kind);
        header.set_mode(0o644);
        header.set_size(bytes.len() as u64);
        // Use the raw header to exercise inputs tar::Builder otherwise rejects.
        header.as_mut_bytes()[..name.len()].copy_from_slice(name.as_bytes());
        header.set_cksum();
        archive.append(&header, *bytes).unwrap();
    }
    archive.into_inner().unwrap()
}

fn package(manifest: &Value, payload: &[u8], extra: &[(&str, EntryType, &[u8])]) -> Vec<u8> {
    let manifest = serde_json::to_vec(manifest).unwrap();
    let mut files = vec![
        ("manifest.json", EntryType::Regular, manifest.as_slice()),
        (ENTRYPOINT, EntryType::Regular, payload),
    ];
    files.extend_from_slice(extra);
    archive(&files)
}

fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
}

fn policy() -> PackageValidator {
    PackageValidator::new(Version::new(2, 1, 0), Version::new(1, 0, 0))
}

fn load(
    policy: &PackageValidator,
    bytes: &[u8],
    format: PackageFormat,
) -> Result<PluginPackage, PackageError> {
    policy.load(TarPackageReader::new(bytes, format))
}

#[test]
fn checks_application_allowlist_and_required_service_versions() {
    let mut declared = manifest();
    declared["compatibility"]["applications"] = json!({"app.one": "^1", "app.two": "^2"});
    declared["compatibility"]["required_services"] = json!({"app.documents": "^2"});
    declared["compatibility"]["optional_features"] = json!(["app.preview"]);
    let bytes = package(&declared, PAYLOAD, &[]);
    let inspect = |validator| load(&validator, &bytes, PackageFormat::Tar);
    assert!(matches!(
        inspect(policy()),
        Err(PackageError::Incompatible("host application ID or version"))
    ));
    assert!(matches!(
        inspect(policy().with_application("app.two", Version::new(1, 0, 0))),
        Err(PackageError::Incompatible("host application ID or version"))
    ));
    assert!(matches!(
        inspect(
            policy()
                .with_application("app.one", Version::new(1, 0, 0))
                .with_host_service("app.documents", Version::new(1, 0, 0))
        ),
        Err(PackageError::Incompatible("required host service"))
    ));
    for (id, version) in [("app.one", 1), ("app.two", 2)] {
        let loaded = inspect(
            policy()
                .with_application(id, Version::new(version, 0, 0))
                .with_host_service("app.documents", Version::new(2, 1, 0))
                .with_host_service("app.documents", Version::new(3, 0, 0)),
        )
        .unwrap();
        assert!(
            loaded
                .manifest()
                .compatibility
                .optional_features
                .contains("app.preview")
        );
        assert_eq!(loaded.available_host_features().len(), 0);
    }
    let loaded = inspect(
        policy()
            .with_application("app.one", Version::new(1, 0, 0))
            .with_host_service("app.documents", Version::new(2, 0, 0))
            .with_host_feature("app.preview")
            .with_host_feature("app.unrequested"),
    )
    .unwrap();
    assert_eq!(
        loaded.available_host_features().collect::<Vec<_>>(),
        ["app.preview"]
    );
}

#[test]
fn reads_valid_tar_and_gzip_and_returns_only_verified_payload() {
    let tar = package(&manifest(), PAYLOAD, &[("./", EntryType::Directory, b"")]);
    let gz = gzip(&tar);
    for (format, bytes) in [(PackageFormat::Tar, tar), (PackageFormat::TarGz, gz)] {
        let package = load(&policy(), &bytes, format).unwrap();
        assert_eq!(package.manifest().id.as_str(), "example.panel");
        assert_eq!(package.entrypoint().path().as_str(), ENTRYPOINT);
        assert_eq!(package.entrypoint().bytes(), PAYLOAD);
        assert_eq!(
            package
                .file(&PackagePath::parse("bin//./panel.wasm").unwrap())
                .unwrap()
                .bytes(),
            PAYLOAD
        );
        assert!(
            package
                .file(&PackagePath::parse("manifest.json").unwrap())
                .is_none()
        );
    }
}

#[test]
fn requires_host_compatibility_capabilities_and_dependencies() {
    let mut manifest = manifest();
    manifest["capabilities"] = json!(["example.views"]);
    manifest["platforms"] = json!([{ "os": "test-os", "arch": "test-arch" }]);
    manifest["dependencies"] = json!([
        { "id": "example.service", "version": "^1.2", "kind": "required" },
        { "id": "example.optional", "version": "1", "kind": "optional" }
    ]);
    let bytes = package(&manifest, PAYLOAD, &[]);
    assert!(matches!(
        load(
            &policy().with_platform("other", "test-arch"),
            &bytes,
            PackageFormat::Tar
        ),
        Err(PackageError::Incompatible(_))
    ));
    assert!(matches!(
        load(
            &policy().with_platform("test-os", "test-arch"),
            &bytes,
            PackageFormat::Tar
        ),
        Err(PackageError::DeniedCapability(_))
    ));
    let host = || {
        policy()
            .with_platform("test-os", "test-arch")
            .allow_capability("example.views")
    };
    let provider = || PluginId::new("example.service").unwrap();
    assert!(matches!(
        load(&host(), &bytes, PackageFormat::Tar),
        Err(PackageError::Dependency(_))
    ));
    assert!(matches!(
        load(
            &host().with_provider(provider(), Version::new(2, 0, 0)),
            &bytes,
            PackageFormat::Tar
        ),
        Err(PackageError::Dependency(_))
    ));
    assert!(
        load(
            &host().with_provider(provider(), Version::new(1, 4, 0)),
            &bytes,
            PackageFormat::Tar
        )
        .is_ok()
    );

    manifest["compatibility"]["plugin_api"] = json!("^3");
    assert!(matches!(
        load(
            &host(),
            &package(&manifest, PAYLOAD, &[]),
            PackageFormat::Tar
        ),
        Err(PackageError::Incompatible("Plugin API version"))
    ));
}

#[test]
fn rejects_bad_hashes_unlisted_files_and_missing_references() {
    let mut manifest = manifest();
    let tampered = package(&manifest, b"not the declared payload", &[]);
    assert!(
        matches!(load(&policy(), &tampered, PackageFormat::Tar), Err(PackageError::HashMismatch(path)) if path == ENTRYPOINT)
    );
    let unlisted = package(
        &manifest,
        PAYLOAD,
        &[("extra", EntryType::Regular, b"data")],
    );
    assert!(
        matches!(load(&policy(), &unlisted, PackageFormat::Tar), Err(PackageError::UnlistedFile(path)) if path == "extra")
    );

    manifest["entrypoint"]["path"] = json!("bin/missing.wasm");
    assert!(
        matches!(load(&policy(), &package(&manifest, PAYLOAD, &[]), PackageFormat::Tar), Err(PackageError::MissingFile(path)) if path == "bin/missing.wasm")
    );
    manifest["entrypoint"]["path"] = json!("../outside.wasm");
    assert!(matches!(
        load(
            &policy(),
            &package(&manifest, PAYLOAD, &[]),
            PackageFormat::Tar
        ),
        Err(PackageError::InvalidPath(_))
    ));
}

#[test]
fn rejects_links_special_paths_and_conflicting_entries() {
    let manifest = manifest();
    for path in ["../outside", "/absolute", "C:/outside", "a\\b"] {
        let data = package(&manifest, PAYLOAD, &[(path, EntryType::Regular, b"")]);
        assert!(
            matches!(
                load(&policy(), &data, PackageFormat::Tar),
                Err(PackageError::InvalidPath(_))
            ),
            "{path}"
        );
    }
    for kind in [EntryType::Symlink, EntryType::Link, EntryType::Fifo] {
        let data = package(&manifest, PAYLOAD, &[("entry", kind, b"")]);
        assert!(
            matches!(
                load(&policy(), &data, PackageFormat::Tar),
                Err(PackageError::UnsupportedEntry(_))
            ),
            "{kind:?}"
        );
    }
    for path in ["./bin/panel.wasm", ENTRYPOINT, "bin/panel.wasm/child"] {
        let data = package(&manifest, PAYLOAD, &[(path, EntryType::Regular, b"")]);
        assert!(
            matches!(
                load(&policy(), &data, PackageFormat::Tar),
                Err(PackageError::DuplicatePath(_))
            ),
            "{path}"
        );
    }
}

#[test]
fn enforces_stream_budgets_and_checks_the_gzip_trailer() {
    let tar = package(&manifest(), PAYLOAD, &[]);
    let mut compressed = gzip(&tar);
    for limits in [
        PackageLimits {
            entries: 1,
            ..PackageLimits::default()
        },
        PackageLimits {
            entry_bytes: 8,
            ..PackageLimits::default()
        },
        PackageLimits {
            expanded_bytes: 512,
            ..PackageLimits::default()
        },
        PackageLimits {
            archive_bytes: 16,
            ..PackageLimits::default()
        },
        PackageLimits {
            manifest_bytes: 16,
            ..PackageLimits::default()
        },
    ] {
        assert!(
            load(
                &policy().with_limits(limits),
                &compressed,
                PackageFormat::TarGz
            )
            .is_err(),
            "{limits:?}"
        );
    }
    let checksum = compressed.len() - 8;
    compressed[checksum] ^= 1;
    assert!(load(&policy(), &compressed, PackageFormat::TarGz).is_err());

    let mut extra = tar;
    extra.extend(b"unexpected data");
    assert!(load(&policy(), &extra, PackageFormat::Tar).is_err());
}

#[test]
fn reports_the_json_decoder_error_as_the_manifest_failure_source() {
    use std::error::Error as _;

    let bytes = archive(&[("manifest.json", EntryType::Regular, b"{\n  broken")]);
    let error = load(&policy(), &bytes, PackageFormat::Tar).unwrap_err();
    assert!(matches!(error, PackageError::Manifest(_)));
    let source = error
        .source()
        .unwrap()
        .downcast_ref::<serde_json::Error>()
        .unwrap();
    assert!(source.is_syntax());
    assert_eq!(source.line(), 2);
}

#[test]
fn rejects_invalid_manifest_schema_identity_and_version_syntax() {
    for (field, value) in [
        ("schema_version", json!(2)),
        ("id", json!("../escape")),
        ("version", json!("not-semver")),
        ("name", json!(" ")),
        ("unknown_field", json!(true)),
    ] {
        let mut manifest = manifest();
        manifest[field] = value;
        let bytes = package(&manifest, PAYLOAD, &[]);
        assert!(
            matches!(
                load(&policy(), &bytes, PackageFormat::Tar),
                Err(PackageError::Manifest(_))
            ),
            "{field}"
        );
    }
}

fn zip_bytes(method: zip::CompressionMethod, entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default().compression_method(method);
    for (name, data) in entries {
        writer.start_file(*name, options).unwrap();
        writer.write_all(data).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

#[test]
fn reads_stored_and_deflated_zip_packages() {
    let manifest = serde_json::to_vec(&manifest()).unwrap();
    for method in [
        zip::CompressionMethod::Stored,
        zip::CompressionMethod::Deflated,
    ] {
        let bytes = zip_bytes(
            method,
            &[("manifest.json", &manifest), (ENTRYPOINT, PAYLOAD)],
        );
        let package = policy()
            .load(ZipPackageReader::new(bytes.as_slice()))
            .unwrap();
        assert_eq!(package.entrypoint().bytes(), PAYLOAD);
        let limits = PackageLimits {
            entries: 1,
            ..PackageLimits::default()
        };
        assert!(matches!(
            policy()
                .with_limits(limits)
                .load(ZipPackageReader::new(bytes.as_slice())),
            Err(PackageError::Limit("entry count"))
        ));
        let limits = PackageLimits {
            expanded_bytes: (manifest.len() + PAYLOAD.len() - 1) as u64,
            ..PackageLimits::default()
        };
        assert!(matches!(
            policy()
                .with_limits(limits)
                .load(ZipPackageReader::new(bytes.as_slice())),
            Err(PackageError::Limit("expanded size"))
        ));
        let limits = PackageLimits {
            archive_bytes: (bytes.len() - 1) as u64,
            ..PackageLimits::default()
        };
        assert!(
            policy()
                .with_limits(limits)
                .load(ZipPackageReader::new(bytes.as_slice()))
                .is_err()
        );
    }
}

#[test]
fn rejects_unsafe_zip_entries() {
    let method = zip::CompressionMethod::Stored;
    let bytes = zip_bytes(method, &[("../escape", b"payload")]);
    assert!(matches!(
        policy().load(ZipPackageReader::new(bytes.as_slice())),
        Err(PackageError::InvalidPath(_))
    ));

    let bytes = zip_bytes(method, &[("bin/panel", b"a"), ("./bin/panel", b"b")]);
    assert!(matches!(
        policy().load(ZipPackageReader::new(bytes.as_slice())),
        Err(PackageError::DuplicatePath(_))
    ));

    // The writer rejects duplicate names, so patch matching local and central names.
    let mut bytes = zip_bytes(method, &[("panel-a", b"a"), ("panel-b", b"b")]);
    while let Some(offset) = bytes.windows(7).position(|name| name == b"panel-b") {
        bytes[offset + 6] = b'a';
    }
    assert!(matches!(
        policy().load(ZipPackageReader::new(bytes.as_slice())),
        Err(PackageError::DuplicatePath(_))
    ));

    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    writer
        .add_symlink("link", "target", zip::write::SimpleFileOptions::default())
        .unwrap();
    let bytes = writer.finish().unwrap().into_inner();
    assert!(matches!(
        policy().load(ZipPackageReader::new(bytes.as_slice())),
        Err(PackageError::UnsupportedEntry(_))
    ));
}

#[test]
fn reads_zstandard_tar_and_checks_the_entire_stream() {
    let tar = package(&manifest(), PAYLOAD, &[]);
    let bytes = zstd::stream::encode_all(tar.as_slice(), 1).unwrap();
    let verified = load(&policy(), &bytes, PackageFormat::TarZst).unwrap();
    assert_eq!(verified.entrypoint().bytes(), PAYLOAD);
    assert!(load(&policy(), &bytes[..bytes.len() - 1], PackageFormat::TarZst).is_err());

    let limits = PackageLimits {
        expanded_bytes: 1024,
        ..PackageLimits::default()
    };
    assert!(load(&policy().with_limits(limits), &bytes, PackageFormat::TarZst).is_err());
}

#[test]
fn rejects_corrupt_zip_content_and_inconsistent_directory_count() {
    let manifest = serde_json::to_vec(&manifest()).unwrap();
    let bytes = zip_bytes(
        zip::CompressionMethod::Stored,
        &[("manifest.json", &manifest), (ENTRYPOINT, PAYLOAD)],
    );

    let mut corrupt = bytes.clone();
    let payload = corrupt
        .windows(PAYLOAD.len())
        .position(|data| data == PAYLOAD)
        .unwrap();
    corrupt[payload] ^= 1;
    assert!(matches!(
        policy().load(ZipPackageReader::new(corrupt.as_slice())),
        Err(PackageError::Io(_))
    ));

    let mut corrupt = bytes;
    let footer = corrupt.len() - 22;
    corrupt[footer + 8..footer + 12].copy_from_slice(&[1, 0, 1, 0]);
    assert!(matches!(
        policy().load(ZipPackageReader::new(corrupt.as_slice())),
        Err(PackageError::Io(_))
    ));
}

#[test]
fn extracts_only_verified_files_to_an_isolated_directory() {
    let resource = "assets/strings/en.txt";
    let contents = b"hello";
    let mut manifest = manifest();
    manifest["resources"] = json!([resource]);
    manifest["files"].as_array_mut().unwrap().push(json!({
        "path": resource,
        "sha256": format!("{:x}", Sha256::digest(contents)),
    }));
    let bytes = package(
        &manifest,
        PAYLOAD,
        &[(resource, EntryType::Regular, contents)],
    );
    let package = load(&policy(), &bytes, PackageFormat::Tar).unwrap();

    let parent = tempfile::tempdir().unwrap();
    let plan = ExtractionPlan::new(&package);
    let directory = plan.extract_in(parent.path()).unwrap();
    let other = plan.extract_in(parent.path()).unwrap();
    assert_ne!(directory.path(), other.path());
    assert!(directory.path().is_absolute());
    assert!(!directory.path().join("manifest.json").exists());
    assert_eq!(
        std::fs::read(directory.entrypoint().path()).unwrap(),
        PAYLOAD
    );
    let entry = directory
        .file(&PackagePath::parse(resource).unwrap())
        .unwrap();
    assert_eq!(std::fs::read(entry.path()).unwrap(), contents);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        for (path, mask) in [
            (directory.path(), 0o700),
            (entry.path().parent().unwrap(), 0o700),
            (entry.path(), 0o600),
        ] {
            let mode = std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode & !mask, 0);
        }
    }

    let path = directory.path().to_owned();
    directory.close().unwrap();
    assert!(!path.exists());
    let path = other.path().to_owned();
    drop(other);
    assert!(!path.exists());
}

#[test]
fn rejects_unsafe_destination_names_and_cleans_up_if_extraction_fails() {
    let parent = tempfile::tempdir().unwrap();
    let oversized = format!("z/{}", "a".repeat(300));
    // The oversized filename exercises rollback after the entrypoint write.
    for name in ["bin/COM¹.txt", oversized.as_str()] {
        let mut manifest = manifest();
        manifest["files"].as_array_mut().unwrap().push(json!({
            "path": name,
            "sha256": format!("{:x}", Sha256::digest(b"extra")),
        }));
        let manifest = serde_json::to_vec(&manifest).unwrap();
        let bytes = zip_bytes(
            zip::CompressionMethod::Stored,
            &[
                ("manifest.json", &manifest),
                (ENTRYPOINT, PAYLOAD),
                (name, b"extra"),
            ],
        );
        let package = policy()
            .load(ZipPackageReader::new(bytes.as_slice()))
            .unwrap();
        let result = ExtractionPlan::new(&package).extract_in(parent.path());
        if name.ends_with(".txt") {
            assert!(matches!(result, Err(PackageError::InvalidPath(path)) if path == name));
        } else {
            assert!(matches!(result, Err(PackageError::Io(_))));
        }
        assert_eq!(parent.path().read_dir().unwrap().count(), 0);
    }
}

#[cfg(any(unix, windows))]
#[test]
fn publishes_installation_and_update_only_on_commit() {
    use obscur_plugin_package::installation::InstallationStore;

    let root = tempfile::tempdir().unwrap();
    let store = InstallationStore::open(root.path()).unwrap();
    let mut metadata = manifest();
    let original = package(&metadata, PAYLOAD, &[]);
    let original = load(&policy(), &original, PackageFormat::Tar).unwrap();
    let id = &original.manifest().id;

    let first = store.stage(&original).unwrap();
    assert!(store.current(id).unwrap().is_none());
    assert_eq!(
        std::fs::read(first.candidate().entrypoint()).unwrap(),
        PAYLOAD
    );
    let old = first.commit().unwrap();
    let reopened = InstallationStore::open(root.path()).unwrap();
    assert_eq!(
        reopened.current(id).unwrap().unwrap().entrypoint(),
        old.entrypoint()
    );

    metadata["version"] = json!("2.0.0");
    let replacement = package(&metadata, PAYLOAD, &[]);
    let replacement = load(&policy(), &replacement, PackageFormat::Tar).unwrap();
    let aborted = reopened.stage(&replacement).unwrap();
    let abandoned = aborted.candidate().payload_dir();
    drop(aborted);
    assert!(!abandoned.exists());
    assert_eq!(
        reopened.current(id).unwrap().unwrap().manifest().version,
        Version::new(1, 3, 0)
    );

    let update = reopened.stage(&replacement).unwrap().commit().unwrap();
    assert_eq!(
        store.current(id).unwrap().unwrap().manifest().version,
        Version::new(2, 0, 0)
    );
    assert_ne!(update.entrypoint(), old.entrypoint());
    assert_eq!(std::fs::read(old.entrypoint()).unwrap(), PAYLOAD);
}

#[cfg(any(unix, windows))]
#[test]
fn rejects_stale_installation_transactions_and_removes_their_files() {
    use obscur_plugin_package::installation::InstallationStore;

    let root = tempfile::tempdir().unwrap();
    let store = InstallationStore::open(root.path()).unwrap();
    let other = InstallationStore::open(root.path()).unwrap();
    let bytes = package(&manifest(), PAYLOAD, &[]);
    let package = load(&policy(), &bytes, PackageFormat::Tar).unwrap();
    let first = store.stage(&package).unwrap();
    let second = other.stage(&package).unwrap();
    let first_path = first.candidate().payload_dir();
    let second_path = second.candidate().payload_dir();

    let barrier = std::sync::Barrier::new(2);
    let (first, second) = std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            barrier.wait();
            first.commit()
        });
        barrier.wait();
        let second = second.commit();
        (first.join().unwrap(), second)
    });
    let (published, rejected, stale_path) = match (first, second) {
        (Ok(published), Err(rejected)) => (published, rejected, second_path),
        (Err(rejected), Ok(published)) => (published, rejected, first_path),
        results => panic!("expected exactly one publication, got {results:?}"),
    };
    assert!(matches!(
        rejected,
        PackageError::Io(error) if error.kind() == std::io::ErrorKind::AlreadyExists
    ));
    assert!(!stale_path.exists());
    let current = other.current(&package.manifest().id).unwrap().unwrap();
    assert_eq!(current.entrypoint(), published.entrypoint());
}

#[cfg(any(unix, windows))]
#[test]
fn invalid_publication_marker_aborts_commit_and_cleans_up_the_candidate() {
    use obscur_plugin_package::installation::InstallationStore;

    let root = tempfile::tempdir().unwrap();
    let store = InstallationStore::open(root.path()).unwrap();
    let bytes = package(&manifest(), PAYLOAD, &[]);
    let package = load(&policy(), &bytes, PackageFormat::Tar).unwrap();
    let installed = store.stage(&package).unwrap().commit().unwrap();
    let staged = store.stage(&package).unwrap();
    let candidate = staged.candidate().payload_dir();

    let marker = root
        .path()
        .join(format!("plugin-{}.current", package.manifest().id.as_str()));
    let invalid = b"invalid generation name";
    std::fs::write(&marker, invalid).unwrap();

    assert!(matches!(
        staged.commit(),
        Err(PackageError::Io(error)) if error.kind() == std::io::ErrorKind::InvalidData
    ));
    assert!(!candidate.exists());
    assert_eq!(std::fs::read(marker).unwrap(), invalid);
    assert_eq!(std::fs::read(installed.entrypoint()).unwrap(), PAYLOAD);
}
