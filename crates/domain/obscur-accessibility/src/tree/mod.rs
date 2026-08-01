#![doc = include_str!("mod.doc/_module.md")]
mod semantic_snapshot;
pub use semantic_snapshot::SemanticSnapshot;
mod semantic_tree;
pub use semantic_tree::SemanticTree;
mod semantic_update;
pub use semantic_update::SemanticUpdate;
