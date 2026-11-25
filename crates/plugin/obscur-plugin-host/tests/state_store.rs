//! Filesystem state storage through the host persistence port.

#![cfg(any(unix, windows))]

use obscur_plugin_host::storage::{
    FilesystemStateStore, PluginStorage, StateStore, StorageNamespace, StorageQuota,
};
use obscur_plugin_model::{identity::PluginId, manifest::Version, state::StateSchema};

fn snapshot(installation: &str, major: u64, payload: &[u8]) -> PluginStorage {
    PluginStorage::new(
        StorageNamespace::new(PluginId::new("example").unwrap(), installation),
        StateSchema::new(Version::new(major, 0, 0)),
        payload.to_vec(),
    )
}

#[test]
fn malformed_schema_preserves_the_decode_error_and_prevents_overwrite() {
    let root = tempfile::tempdir().unwrap();
    let store = FilesystemStateStore::open(root.path(), StorageQuota::new(16)).unwrap();
    let candidate = snapshot("install", 1, b"state");
    assert!(store.compare_exchange(None, &candidate).unwrap());
    let path = root
        .path()
        .read_dir()
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.extension().is_some_and(|ext| ext == "state"))
        .unwrap();
    let malformed = b"obscur-state-1\n\xff\nstate";
    std::fs::write(&path, malformed).unwrap();

    let error = store.compare_exchange(None, &candidate).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.get_ref().unwrap().is::<std::str::Utf8Error>());
    assert_eq!(std::fs::read(path).unwrap(), malformed);
}

#[test]
fn reopens_versioned_state_and_enforces_namespace_and_quota() {
    let root = tempfile::tempdir().unwrap();
    let store = FilesystemStateStore::open(root.path(), StorageQuota::new(4)).unwrap();
    let first = snapshot("../first", 1, b"a\0\nb");
    let second = snapshot("second", 1, b"");
    assert!(store.compare_exchange(None, &first).unwrap());
    assert!(store.compare_exchange(None, &second).unwrap());

    let store = FilesystemStateStore::open(root.path(), StorageQuota::new(4)).unwrap();
    assert_eq!(
        store.load(first.namespace()).unwrap().as_ref(),
        Some(&first)
    );
    assert_eq!(
        store.load(second.namespace()).unwrap().as_ref(),
        Some(&second)
    );
    assert!(store.compare_exchange(Some(&first), &second).is_err());

    let oversized = snapshot("../first", 2, b"longer");
    assert!(store.compare_exchange(Some(&first), &oversized).is_err());
    assert_eq!(store.load(first.namespace()).unwrap(), Some(first.clone()));

    let stricter = FilesystemStateStore::open(root.path(), StorageQuota::new(3)).unwrap();
    assert_eq!(
        stricter.load(first.namespace()).unwrap_err().kind(),
        std::io::ErrorKind::InvalidData
    );
}

#[test]
fn concurrent_migrations_cannot_overwrite_each_other() {
    let root = tempfile::tempdir().unwrap();
    let first = FilesystemStateStore::open(root.path(), StorageQuota::new(16)).unwrap();
    let second = FilesystemStateStore::open(root.path(), StorageQuota::new(16)).unwrap();
    let old = snapshot("install", 1, b"old");
    let left = snapshot("install", 2, b"left");
    let right = snapshot("install", 2, b"right");
    assert!(first.compare_exchange(None, &old).unwrap());
    let barrier = std::sync::Barrier::new(2);
    let (left_won, right_won) = std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            barrier.wait();
            first.compare_exchange(Some(&old), &left).unwrap()
        });
        barrier.wait();
        let right_won = second.compare_exchange(Some(&old), &right).unwrap();
        (worker.join().unwrap(), right_won)
    });
    assert_ne!(left_won, right_won);
    let winner = if left_won { &left } else { &right };
    assert_eq!(second.load(old.namespace()).unwrap().as_ref(), Some(winner));
}

#[test]
fn migration_commit_updates_memory_only_after_publishing() {
    use obscur_plugin_host::update::StateMigration;
    use obscur_plugin_model::{manifest::VersionReq, state::MigrationDescriptor};

    let root = tempfile::tempdir().unwrap();
    let store = FilesystemStateStore::open(root.path(), StorageQuota::new(16)).unwrap();
    let mut current = snapshot("install", 1, b"old");
    assert!(store.compare_exchange(None, &current).unwrap());
    let descriptor = MigrationDescriptor::new(
        VersionReq::parse("^1").unwrap(),
        StateSchema::new(Version::new(2, 0, 0)),
    );
    let migrated =
        StateMigration::stage(&mut current, &descriptor, |_| Ok(b"new".to_vec())).unwrap();
    let concurrent = snapshot("install", 1, b"concurrent");
    assert!(
        store
            .compare_exchange(Some(&snapshot("install", 1, b"old")), &concurrent)
            .unwrap()
    );
    assert!(!migrated.commit_to(&store).unwrap());
    assert_eq!(current.bytes(), b"old");

    current = store.load(current.namespace()).unwrap().unwrap();
    let migrated =
        StateMigration::stage(&mut current, &descriptor, |_| Ok(b"new".to_vec())).unwrap();
    assert!(migrated.commit_to(&store).unwrap());
    assert_eq!(current.bytes(), b"new");
    assert_eq!(
        store.load(current.namespace()).unwrap(),
        Some(current.clone())
    );
}

#[test]
fn keeps_existing_global_files_and_evicts_only_the_selected_cache() {
    use obscur_plugin_host::storage::CacheStore;
    use obscur_plugin_model::state::StateScope;

    let root = tempfile::tempdir().unwrap();
    // A global snapshot written before storage scopes were introduced.
    let legacy = "b4c4a75b72568b1786440bdcab1157a1377f2bf21a3fd97f6de9e67bcf7ad821.state";
    std::fs::write(root.path().join(legacy), b"obscur-state-1\n1.0.0\nglobal").unwrap();
    let store = FilesystemStateStore::open(root.path(), StorageQuota::new(16)).unwrap();
    let global = snapshot("install", 1, b"global");
    let scoped = |installation, scope, bytes: &[u8]| {
        let namespace = StorageNamespace::new(PluginId::new("example").unwrap(), installation);
        PluginStorage::new(
            namespace.with_scope(scope),
            global.schema().clone(),
            bytes.to_vec(),
        )
    };
    let first = scoped("install", StateScope::Instance("../first".into()), b"first");
    let second = scoped("install", StateScope::Instance("second".into()), b"second");
    let cache = scoped("install", StateScope::Cache, b"cache");
    let other_cache = scoped("different", StateScope::Cache, b"other");
    assert_eq!(
        store.load(global.namespace()).unwrap().as_ref(),
        Some(&global)
    );
    for value in [&first, &second, &cache, &other_cache] {
        assert!(store.compare_exchange(None, value).unwrap());
    }
    assert!(store.compare_exchange(Some(&global), &first).is_err());

    assert!(store.clear_cache(first.namespace()).unwrap());
    assert!(!store.clear_cache(global.namespace()).unwrap());
    assert!(store.load(cache.namespace()).unwrap().is_none());
    assert!(!store.compare_exchange(Some(&cache), &cache).unwrap());
    for value in [&global, &first, &second, &other_cache] {
        assert_eq!(store.load(value.namespace()).unwrap().as_ref(), Some(value));
    }
}
