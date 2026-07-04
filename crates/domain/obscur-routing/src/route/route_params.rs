//! Typed parameters captured from a route pattern match (spec §6.13).

use std::collections::BTreeMap;
use std::collections::btree_map;
use std::str::FromStr;

/// The named parameters captured when a [`RoutePattern`] matches a path
/// (spec §6.13).
///
/// Parameters are stored as string key/value pairs in a [`BTreeMap`], which
/// keeps iteration in deterministic (sorted-by-key) order regardless of capture
/// order. The strings are the raw path segments captured by `:name` and
/// `*name` pattern segments.
///
/// The "typed params" feature of §6.13 is provided by
/// [`get_parsed`](RouteParams::get_parsed), which parses a captured string into
/// any [`FromStr`] target on demand.
///
/// [`RoutePattern`]: crate::route::RoutePattern
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RouteParams {
    /// The captured parameters, keyed by name for deterministic iteration.
    values: BTreeMap<Box<str>, Box<str>>,
}

impl RouteParams {
    /// Creates an empty set of parameters.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a parameter, replacing any existing value for the same key.
    pub fn insert(&mut self, key: impl Into<Box<str>>, value: impl Into<Box<str>>) {
        self.values.insert(key.into(), value.into());
    }

    /// Returns the raw string value for `key`, if present.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|v| &**v)
    }

    /// Returns the value for `key` parsed into `T` via [`FromStr`] (spec §6.13
    /// typed params).
    ///
    /// Returns `None` when the key is absent *or* when the stored string fails
    /// to parse as `T`; the two failure modes are intentionally
    /// indistinguishable, since a caller asking for a typed value only cares
    /// whether a valid `T` is available.
    #[must_use]
    pub fn get_parsed<T: FromStr>(&self, key: &str) -> Option<T> {
        self.get(key).and_then(|v| v.parse().ok())
    }

    /// Returns whether a parameter named `key` is present.
    #[must_use]
    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// Returns the number of captured parameters.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether no parameters are captured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns an iterator over the parameters in sorted key order.
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
        // Success.
        assert_eq!(params.get_parsed::<u32>("id"), Some(42));
        // Parse failure returns None.
        assert_eq!(params.get_parsed::<u32>("bad"), None);
        // Absent key returns None.
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
