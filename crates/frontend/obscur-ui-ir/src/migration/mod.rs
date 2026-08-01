#![doc = include_str!("mod.doc/_module.md")]
mod migration_registry;
pub use migration_registry::{MigrationError, MigrationRegistry};
mod ui_ir_migration;
pub use ui_ir_migration::{IdentityMigration, UiIrMigration};
