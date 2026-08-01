#![doc = include_str!("ui_ir_document.doc/_module.md")]

use obscur_capability::CapabilityId;

use crate::document::UiIrNode;
use crate::source::SourceMap;
use crate::version::UiIrVersion;

#[doc = include_str!("ui_ir_document.doc/UiIrDocument.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct UiIrDocument {
    version: UiIrVersion,
    root: UiIrNode,
    source_map: SourceMap,
    required_capabilities: Vec<CapabilityId>,
}

impl UiIrDocument {
    #[doc = include_str!("ui_ir_document.doc/new.md")]
    #[must_use]
    pub fn new(root: UiIrNode) -> Self {
        Self {
            version: UiIrVersion::CURRENT,
            root,
            source_map: SourceMap::new(),
            required_capabilities: Vec::new(),
        }
    }

    #[doc = include_str!("ui_ir_document.doc/with_version.md")]
    #[must_use]
    pub const fn with_version(mut self, version: UiIrVersion) -> Self {
        self.version = version;
        self
    }

    #[doc = include_str!("ui_ir_document.doc/with_source_map.md")]
    #[must_use]
    pub fn with_source_map(mut self, source_map: SourceMap) -> Self {
        self.source_map = source_map;
        self
    }

    #[doc = include_str!("ui_ir_document.doc/requiring.md")]
    #[must_use]
    pub fn requiring(mut self, capability: CapabilityId) -> Self {
        self.required_capabilities.push(capability);
        self
    }

    #[doc = include_str!("ui_ir_document.doc/version.md")]
    #[must_use]
    pub const fn version(&self) -> UiIrVersion {
        self.version
    }

    #[doc = include_str!("ui_ir_document.doc/root.md")]
    #[must_use]
    pub const fn root(&self) -> &UiIrNode {
        &self.root
    }

    #[doc = include_str!("ui_ir_document.doc/root_mut.md")]
    #[must_use]
    pub fn root_mut(&mut self) -> &mut UiIrNode {
        &mut self.root
    }

    #[doc = include_str!("ui_ir_document.doc/source_map.md")]
    #[must_use]
    pub const fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    #[doc = include_str!("ui_ir_document.doc/required_capabilities.md")]
    #[must_use]
    pub fn required_capabilities(&self) -> &[CapabilityId] {
        &self.required_capabilities
    }

    #[doc = include_str!("ui_ir_document.doc/node_count.md")]
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.root.node_count()
    }

    #[doc = include_str!("ui_ir_document.doc/find.md")]
    #[must_use]
    pub fn find(&self, id: u64) -> Option<&UiIrNode> {
        self.root.find(id)
    }

    #[doc = include_str!("ui_ir_document.doc/requires.md")]
    #[must_use]
    pub fn requires(&self, cap: &CapabilityId) -> bool {
        self.required_capabilities.contains(cap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_uses_current_version_and_no_capabilities() {
        let doc = UiIrDocument::new(UiIrNode::element(1, "div"));
        assert_eq!(doc.version(), UiIrVersion::CURRENT);
        assert!(doc.required_capabilities().is_empty());
        assert!(doc.source_map().is_empty());
        assert_eq!(doc.root().id(), 1);
    }

    #[test]
    fn requiring_and_requires() {
        let cap = CapabilityId::new("fs.read");
        let doc = UiIrDocument::new(UiIrNode::fragment(1)).requiring(cap.clone());
        assert!(doc.requires(&cap));
        assert!(!doc.requires(&CapabilityId::new("net.fetch")));
        assert_eq!(doc.required_capabilities(), &[cap]);
    }

    #[test]
    fn node_count_and_find_delegate_to_root() {
        let root = UiIrNode::element(1, "div")
            .with_child(UiIrNode::text(2, "a"))
            .with_child(UiIrNode::text(3, "b"));
        let doc = UiIrDocument::new(root);
        assert_eq!(doc.node_count(), 3);
        assert_eq!(doc.find(2).map(UiIrNode::id), Some(2));
        assert!(doc.find(99).is_none());
    }

    #[test]
    fn with_version_overrides_default() {
        let doc = UiIrDocument::new(UiIrNode::fragment(1)).with_version(UiIrVersion::new(2, 3));
        assert_eq!(doc.version(), UiIrVersion::new(2, 3));
    }

    #[test]
    fn root_mut_allows_mutation() {
        let mut doc = UiIrDocument::new(UiIrNode::element(1, "div"));
        *doc.root_mut() = UiIrNode::element(1, "span");
        assert_eq!(doc.root().tag(), "span");
    }
}
