use crate::document::UiIrDocument;
use crate::version::UiIrVersion;

use super::UiIrMigration;

#[doc = include_str!("../../docs/api/migration/ui_ir_migration.doc/IdentityMigration.md")]
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
