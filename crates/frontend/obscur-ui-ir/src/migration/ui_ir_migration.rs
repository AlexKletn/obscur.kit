//! `UiIrMigration`: an object-safe transform from one IR version to the next
//! (spec §2.5 migrations).

use crate::document::UiIrDocument;
use crate::version::UiIrVersion;

/// A single version-to-version transform over a UI IR document (spec §2.5).
///
/// A migration knows the [`from_version`](UiIrMigration::from_version) it
/// consumes and the [`to_version`](UiIrMigration::to_version) it produces, and
/// rewrites a document from the former schema to the latter via
/// [`migrate`](UiIrMigration::migrate) (which takes the document by value and
/// returns the migrated document). Real migrations override `migrate` to
/// rewrite the node tree and its properties for the new schema; see
/// [`IdentityMigration`] for the trivial version-bump-only reference.
///
/// The trait is object-safe — migrations are stored as `Box<dyn UiIrMigration>`
/// in a [`MigrationRegistry`](crate::migration::MigrationRegistry) — and
/// requires a [`std::fmt::Debug`] supertrait so a boxed migration is itself
/// `Debug`.
pub trait UiIrMigration: std::fmt::Debug {
    /// Returns the IR version this migration consumes.
    fn from_version(&self) -> UiIrVersion;

    /// Returns the IR version this migration produces.
    fn to_version(&self) -> UiIrVersion;

    /// Rewrites `doc` from [`from_version`](UiIrMigration::from_version) to
    /// [`to_version`](UiIrMigration::to_version), returning the migrated
    /// document.
    fn migrate(&self, doc: UiIrDocument) -> UiIrDocument;
}

/// The trivial reference migration: a pure version bump that leaves the tree
/// untouched (spec §2.5).
///
/// [`migrate`](UiIrMigration::migrate) returns the document unchanged except for
/// stamping it with [`to`](IdentityMigration::to). It is useful for versions
/// whose schema did not actually change, and as a template real migrations
/// build on by overriding `migrate` to rewrite nodes and properties.
#[derive(Debug, Clone)]
pub struct IdentityMigration {
    from: UiIrVersion,
    to: UiIrVersion,
}

impl IdentityMigration {
    /// Creates an identity migration from `from` to `to`.
    #[must_use]
    pub const fn new(from: UiIrVersion, to: UiIrVersion) -> Self {
        Self { from, to }
    }
}

impl UiIrMigration for IdentityMigration {
    fn from_version(&self) -> UiIrVersion {
        self.from
    }

    fn to_version(&self) -> UiIrVersion {
        self.to
    }

    fn migrate(&self, doc: UiIrDocument) -> UiIrDocument {
        doc.with_version(self.to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::UiIrNode;

    #[test]
    fn identity_migration_reports_versions() {
        let migration = IdentityMigration::new(UiIrVersion::new(1, 0), UiIrVersion::new(1, 1));
        assert_eq!(migration.from_version(), UiIrVersion::new(1, 0));
        assert_eq!(migration.to_version(), UiIrVersion::new(1, 1));
    }

    #[test]
    fn identity_migration_only_bumps_version() {
        let migration = IdentityMigration::new(UiIrVersion::new(1, 0), UiIrVersion::new(1, 1));
        let doc = UiIrDocument::new(UiIrNode::element(1, "div").with_child(UiIrNode::text(2, "x")));
        let migrated = migration.migrate(doc.clone());
        assert_eq!(migrated.version(), UiIrVersion::new(1, 1));
        // The tree is identical apart from the version.
        assert_eq!(migrated.root(), doc.root());
        assert_eq!(migrated.node_count(), doc.node_count());
    }

    #[test]
    fn migration_is_object_safe() {
        let _: Box<dyn UiIrMigration> = Box::new(IdentityMigration::new(
            UiIrVersion::new(1, 0),
            UiIrVersion::new(1, 1),
        ));
    }
}
