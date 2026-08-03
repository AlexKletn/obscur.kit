#![doc = include_str!("route_params.doc/_module.md")]

use std::collections::BTreeMap;
use std::collections::btree_map;
use std::str::FromStr;

#[doc = include_str!("route_params.doc/RouteParams.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RouteParams {
    #[doc = include_str!("route_params.doc/values.md")]
    values: BTreeMap<Box<str>, Box<str>>,
}

impl RouteParams {
    #[doc = include_str!("route_params.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("route_params.doc/insert.md")]
    pub fn insert(&mut self, key: impl Into<Box<str>>, value: impl Into<Box<str>>) {
        self.values.insert(key.into(), value.into());
    }

    #[doc = include_str!("route_params.doc/get.md")]
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|v| &**v)
    }

    #[doc = include_str!("route_params.doc/get_parsed.md")]
    #[must_use]
    pub fn get_parsed<T: FromStr>(&self, key: &str) -> Option<T> {
        self.get(key).and_then(|v| v.parse().ok())
    }

    #[doc = include_str!("route_params.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    #[doc = include_str!("route_params.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[doc = include_str!("route_params.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    #[doc = include_str!("route_params.doc/iter.md")]
    pub fn iter(&self) -> btree_map::Iter<'_, Box<str>, Box<str>> {
        self.values.iter()
    }
}

impl<'a> IntoIterator for &'a RouteParams {
    type Item = (&'a Box<str>, &'a Box<str>);
    type IntoIter = btree_map::Iter<'a, Box<str>, Box<str>>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut params = RouteParams::new();
        assert!(params.is_empty());
        params.insert("id", "42");
        params.insert("name", "ada");
        assert_eq!(params.len(), 2);
        assert!(!params.is_empty());
        assert_eq!(params.get("id"), Some("42"));
        assert_eq!(params.get("name"), Some("ada"));
        assert_eq!(params.get("missing"), None);
        assert!(params.contains("id"));
        assert!(!params.contains("missing"));
    }

    #[test]
    fn get_parsed_typed() {
        let mut params = RouteParams::new();
        params.insert("id", "42");
        params.insert("bad", "not-a-number");
        assert_eq!(params.get_parsed::<u32>("id"), Some(42));
        assert_eq!(params.get_parsed::<u32>("bad"), None);
        assert_eq!(params.get_parsed::<u32>("missing"), None);
    }

    #[test]
    fn iter_is_sorted_by_key() {
        let mut params = RouteParams::new();
        params.insert("z", "1");
        params.insert("a", "2");
        let keys: Vec<&str> = params.iter().map(|(k, _)| &**k).collect();
        assert_eq!(keys, vec!["a", "z"]);
    }

    #[test]
    fn insert_replaces_existing() {
        let mut params = RouteParams::new();
        params.insert("id", "1");
        params.insert("id", "2");
        assert_eq!(params.len(), 1);
        assert_eq!(params.get("id"), Some("2"));
    }
}
