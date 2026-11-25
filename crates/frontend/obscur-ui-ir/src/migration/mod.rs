//! Version-to-version migration of UI IR documents: a [`MigrationRegistry`]
//! chains [`UiIrMigration`]s (with [`IdentityMigration`] as the trivial
//! reference) to bring a document up to a target version, surfacing
//! [`MigrationError`] when no path exists (spec §2.5 migrations).
mod migration_registry;
pub use migration_registry::MigrationRegistry;
mod migration_error;
pub use migration_error::MigrationError;
mod ui_ir_migration;
pub use ui_ir_migration::UiIrMigration;
mod identity_migration;
pub use identity_migration::IdentityMigration;
