//! State migration through the plugin host API.

use obscur_plugin_api::status::PluginApiError;
use obscur_plugin_host::{
    storage::{PluginStorage, StorageNamespace},
    update::{StateMigration, StateMigrationError},
};
use obscur_plugin_model::{
    identity::PluginId,
    manifest::{Version, VersionReq},
    state::{MigrationDescriptor, StateSchema},
};

fn current() -> PluginStorage {
    PluginStorage::new(
        StorageNamespace::new(PluginId::new("example").unwrap(), "installation"),
        StateSchema::new(Version::new(1, 2, 0)),
        b"old".to_vec(),
    )
}

fn descriptor(source: &str) -> MigrationDescriptor {
    MigrationDescriptor::new(
        VersionReq::parse(source).unwrap(),
        StateSchema::new(Version::new(2, 0, 0)),
    )
}

#[test]
fn stages_separate_state_and_adopts_it_only_after_confirmation() {
    let mut storage = current();
    let backup = storage.clone();
    let migrate = |source: &[u8]| Ok([source, b"+new"].concat());
    {
        let trial = StateMigration::stage(&mut storage, &descriptor("^1"), migrate).unwrap();
        assert_eq!(trial.candidate().namespace(), backup.namespace());
        assert_eq!(trial.candidate().schema().version(), &Version::new(2, 0, 0));
        assert_eq!(trial.candidate().bytes(), b"old+new");
    }
    assert_eq!(storage, backup);

    StateMigration::stage(&mut storage, &descriptor("^1"), migrate)
        .unwrap()
        .commit();
    assert_eq!(storage.bytes(), b"old+new");
    assert_eq!(storage.schema().version(), &Version::new(2, 0, 0));
    assert_eq!(backup.bytes(), b"old");
}

#[test]
fn rejects_an_incompatible_schema_without_running_the_migrator() {
    let mut storage = current();
    let called = std::cell::Cell::new(false);
    let result = StateMigration::stage(&mut storage, &descriptor("^2"), |_| {
        called.set(true);
        Ok(Vec::new())
    });
    assert!(matches!(
        result,
        Err(StateMigrationError::IncompatibleSource { found, accepted })
            if found == Version::new(1, 2, 0) && accepted == VersionReq::parse("^2").unwrap()
    ));
    assert!(!called.get());
    assert_eq!(storage, current());
}

#[test]
fn failed_or_panicking_migration_preserves_the_original_state() {
    let mut storage = current();
    let error = PluginApiError::Rejected("invalid old state".into());
    assert!(matches!(
        StateMigration::stage(&mut storage, &descriptor("^1"), |_| Err(error.clone())),
        Err(StateMigrationError::Callback(found)) if found == error
    ));
    assert!(matches!(
        StateMigration::stage(&mut storage, &descriptor("^1"), |_| panic!(
            "migration panicked"
        )),
        Err(StateMigrationError::Callback(PluginApiError::Panicked))
    ));
    assert_eq!(storage, current());
}
