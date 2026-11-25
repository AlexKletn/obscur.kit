//! `MigrationRegistry`: chains registered migrations to bring a document up to a
//! target version (spec §2.5 migrations).

use crate::document::UiIrDocument;
use crate::migration::UiIrMigration;
use crate::version::UiIrVersion;

use super::MigrationError;

#[doc = include_str!("../../docs/api/migration/migration_registry.doc/MigrationRegistry.md")]
#[derive(Debug, Default)]
pub struct MigrationRegistry {
    migrations: Vec<Box<dyn UiIrMigration>>,
}

impl MigrationRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    /// Registers `migration`.
    pub fn register(&mut self, migration: Box<dyn UiIrMigration>) {
        self.migrations.push(migration);
    }

    /// Returns the number of registered migrations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.migrations.len()
    }

    /// Returns whether no migrations are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.migrations.is_empty()
    }

    /// Finds a registered migration whose `from_version` matches `version`.
    fn migration_from(&self, version: UiIrVersion) -> Option<&dyn UiIrMigration> {
        self.migrations
            .iter()
            .map(AsRef::as_ref)
            .find(|migration| migration.from_version() == version)
    }

    #[doc = include_str!("../../docs/api/migration/migration_registry.doc/migrate.md")]
    pub fn migrate(
        &self,
        doc: UiIrDocument,
        target: UiIrVersion,
    ) -> Result<UiIrDocument, MigrationError> {
        let mut doc = doc;
        // Bound iterations to guard against cyclic / non-advancing chains.
        let max_steps = self.migrations.len() + 1;
        for _ in 0..max_steps {
            let current = doc.version();
            if current == target {
                return Ok(doc);
            }
            match self.migration_from(current) {
                Some(migration) => doc = migration.migrate(doc),
                None => {
                    return Err(MigrationError::NoPath {
                        from: current,
                        to: target,
                    });
                }
            }
        }
        // Ran out of steps without hitting the target: no viable path.
        Err(MigrationError::NoPath {
            from: doc.version(),
            to: target,
        })
    }

    /// Returns whether a chain of registered migrations reaches `to` from
    /// `from`, without applying anything.
    #[must_use]
    pub fn can_migrate(&self, from: UiIrVersion, to: UiIrVersion) -> bool {
        let mut current = from;
        let max_steps = self.migrations.len() + 1;
        for _ in 0..max_steps {
            if current == to {
                return true;
            }
            match self.migration_from(current) {
                Some(migration) => current = migration.to_version(),
                None => return false,
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::UiIrNode;
    use crate::migration::IdentityMigration;

    fn registry_1_0_to_1_2() -> MigrationRegistry {
        let mut registry = MigrationRegistry::new();
        registry.register(Box::new(IdentityMigration::new(
            UiIrVersion::new(1, 0),
            UiIrVersion::new(1, 1),
        )));
        registry.register(Box::new(IdentityMigration::new(
            UiIrVersion::new(1, 1),
            UiIrVersion::new(1, 2),
        )));
        registry
    }

    #[test]
    fn len_and_is_empty() {
        let mut registry = MigrationRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        registry.register(Box::new(IdentityMigration::new(
            UiIrVersion::new(1, 0),
            UiIrVersion::new(1, 1),
        )));
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn migrate_chains_two_steps() {
        let registry = registry_1_0_to_1_2();
        let doc = UiIrDocument::new(UiIrNode::element(1, "div")); // starts at CURRENT == 1.0
        let migrated = registry
            .migrate(doc, UiIrVersion::new(1, 2))
            .expect("chain should reach 1.2");
        assert_eq!(migrated.version(), UiIrVersion::new(1, 2));
    }

    #[test]
    fn migrate_unreachable_target_is_no_path() {
        let registry = registry_1_0_to_1_2();
        let doc = UiIrDocument::new(UiIrNode::element(1, "div"));
        let err = registry
            .migrate(doc, UiIrVersion::new(2, 0))
            .expect_err("no migration reaches 2.0");
        assert_eq!(
            err,
            MigrationError::NoPath {
                from: UiIrVersion::new(1, 2),
                to: UiIrVersion::new(2, 0),
            }
        );
    }

    #[test]
    fn migrate_already_at_target_is_unchanged() {
        let registry = registry_1_0_to_1_2();
        let doc = UiIrDocument::new(UiIrNode::element(1, "div"));
        let migrated = registry
            .migrate(doc.clone(), UiIrVersion::CURRENT)
            .expect("already at target");
        assert_eq!(migrated, doc);
    }

    #[test]
    fn can_migrate_reports_reachability() {
        let registry = registry_1_0_to_1_2();
        assert!(registry.can_migrate(UiIrVersion::new(1, 0), UiIrVersion::new(1, 2)));
        assert!(registry.can_migrate(UiIrVersion::new(1, 1), UiIrVersion::new(1, 2)));
        assert!(!registry.can_migrate(UiIrVersion::new(1, 0), UiIrVersion::new(2, 0)));
    }

    #[test]
    fn cyclic_migration_is_bounded() {
        // A self-loop (to == from) must not spin forever.
        let mut registry = MigrationRegistry::new();
        registry.register(Box::new(IdentityMigration::new(
            UiIrVersion::new(1, 0),
            UiIrVersion::new(1, 0),
        )));
        let doc = UiIrDocument::new(UiIrNode::element(1, "div"));
        let err = registry
            .migrate(doc, UiIrVersion::new(9, 9))
            .expect_err("cycle cannot reach target");
        assert_eq!(
            err,
            MigrationError::NoPath {
                from: UiIrVersion::new(1, 0),
                to: UiIrVersion::new(9, 9),
            }
        );
        assert!(!registry.can_migrate(UiIrVersion::new(1, 0), UiIrVersion::new(9, 9)));
    }

    #[test]
    fn migration_error_display() {
        let err = MigrationError::NoPath {
            from: UiIrVersion::new(1, 0),
            to: UiIrVersion::new(2, 0),
        };
        assert_eq!(err.to_string(), "no migration path from 1.0 to 2.0");
    }
}
