//! `UiIrMigration`: an object-safe transform from one IR version to the next
//! (spec §2.5 migrations).

use crate::document::UiIrDocument;
use crate::version::UiIrVersion;

#[doc = include_str!("../../docs/api/migration/ui_ir_migration.doc/UiIrMigration.md")]
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
