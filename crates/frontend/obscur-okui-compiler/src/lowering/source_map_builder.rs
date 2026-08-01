#![doc = include_str!("source_map_builder.doc/_module.md")]

use obscur_ui_ir::source::{SourceLocation, SourceMap, SourceReference};

#[doc = include_str!("source_map_builder.doc/SourceMapBuilder.md")]
#[derive(Debug, Clone)]
pub struct SourceMapBuilder {
    source_name: Box<str>,
    map: SourceMap,
}

impl SourceMapBuilder {
    #[doc = include_str!("source_map_builder.doc/new.md")]
    #[must_use]
    pub fn new(source_name: impl Into<Box<str>>) -> Self {
        Self {
            source_name: source_name.into(),
            map: SourceMap::new(),
        }
    }

    #[doc = include_str!("source_map_builder.doc/record.md")]
    pub fn record(&mut self, node_id: u64, offset: u32) {
        let location = SourceLocation::new(offset, 1, 1);
        self.map.insert(
            node_id,
            SourceReference::new(self.source_name.clone(), location, location),
        );
    }

    #[doc = include_str!("source_map_builder.doc/build.md")]
    #[must_use]
    pub fn build(self) -> SourceMap {
        self.map
    }

    #[doc = include_str!("source_map_builder.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[doc = include_str!("source_map_builder.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl Default for SourceMapBuilder {
    #[doc = include_str!("source_map_builder.doc/default.md")]
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_then_build_maps_ids_to_offsets() {
        let mut builder = SourceMapBuilder::new("app.okui");
        assert!(builder.is_empty());
        builder.record(1, 0);
        builder.record(2, 12);
        assert_eq!(builder.len(), 2);
        assert!(!builder.is_empty());

        let map = builder.build();
        assert_eq!(map.len(), 2);
        let reference = map.get(2).expect("node 2 recorded");
        assert_eq!(reference.source(), "app.okui");
        assert_eq!(reference.start().offset(), 12);
        assert!(reference.is_empty());
    }

    #[test]
    fn default_uses_empty_source_name() {
        let mut builder = SourceMapBuilder::default();
        builder.record(1, 3);
        let map = builder.build();
        assert_eq!(map.get(1).expect("node 1").source(), "");
    }
}
