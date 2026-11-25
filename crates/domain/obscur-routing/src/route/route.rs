//! A parsed URL-like location: path, query, and fragment (spec §6.13).

#[doc = include_str!("../../docs/api/route/route.doc/Route.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    /// The normalized path; always starts with `/`.
    path: Box<str>,
    /// The ordered query key/value pairs, in the order they appeared.
    query: Vec<(Box<str>, Box<str>)>,
    /// The fragment (the part after `#`), without the leading `#`.
    fragment: Option<Box<str>>,
}

impl Route {
    #[doc = include_str!("../../docs/api/route/route.doc/new.md")]
    #[must_use]
    pub fn new(path: impl Into<Box<str>>) -> Self {
        Self {
            path: normalize_path(&path.into()),
            query: Vec::new(),
            fragment: None,
        }
    }

    #[doc = include_str!("../../docs/api/route/route.doc/parse.md")]
    #[must_use]
    pub fn parse(input: &str) -> Self {
        let (before_fragment, fragment) = match input.split_once('#') {
            Some((head, frag)) => (head, Some(Box::<str>::from(frag))),
            None => (input, None),
        };

        let (path_part, query_part) = match before_fragment.split_once('?') {
            Some((head, query)) => (head, Some(query)),
            None => (before_fragment, None),
        };

        let query = query_part.map_or_else(Vec::new, parse_query);

        Self {
            path: normalize_path(path_part),
            query,
            fragment,
        }
    }

    /// Returns the normalized path (always starting with `/`).
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the ordered query key/value pairs.
    #[must_use]
    pub fn query(&self) -> &[(Box<str>, Box<str>)] {
        &self.query
    }

    /// Returns the value of the first query parameter matching `key`, if any.
    #[must_use]
    pub fn query_param(&self, key: &str) -> Option<&str> {
        self.query
            .iter()
            .find(|(k, _)| &**k == key)
            .map(|(_, v)| &**v)
    }

    /// Returns the fragment (without the leading `#`), if present.
    #[must_use]
    pub fn fragment(&self) -> Option<&str> {
        self.fragment.as_deref()
    }

    #[doc = include_str!("../../docs/api/route/route.doc/segments.md")]
    #[must_use]
    pub fn segments(&self) -> Vec<&str> {
        self.path.split('/').filter(|s| !s.is_empty()).collect()
    }

    #[doc = include_str!("../../docs/api/route/route.doc/to_string_path.md")]
    #[must_use]
    pub fn to_string_path(&self) -> String {
        let mut out = String::from(&*self.path);
        if !self.query.is_empty() {
            out.push('?');
            for (i, (k, v)) in self.query.iter().enumerate() {
                if i > 0 {
                    out.push('&');
                }
                out.push_str(k);
                out.push('=');
                out.push_str(v);
            }
        }
        if let Some(fragment) = &self.fragment {
            out.push('#');
            out.push_str(fragment);
        }
        out
    }
}

/// Normalizes a raw path so that it begins with `/`.
fn normalize_path(path: &str) -> Box<str> {
    if path.starts_with('/') {
        Box::from(path)
    } else {
        let mut normalized = String::with_capacity(path.len() + 1);
        normalized.push('/');
        normalized.push_str(path);
        normalized.into_boxed_str()
    }
}

/// Parses a raw query string (`k=v&k2=v2`) into ordered key/value pairs.
fn parse_query(query: &str) -> Vec<(Box<str>, Box<str>)> {
    query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| match pair.split_once('=') {
            Some((k, v)) => (Box::from(k), Box::from(v)),
            None => (Box::from(pair), Box::from("")),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_location() {
        let route = Route::parse("/a/b?x=1&y=2#frag");
        assert_eq!(route.path(), "/a/b");
        assert_eq!(route.query_param("x"), Some("1"));
        assert_eq!(route.query_param("y"), Some("2"));
        assert_eq!(route.query_param("z"), None);
        assert_eq!(route.fragment(), Some("frag"));
        assert_eq!(route.segments(), vec!["a", "b"]);
        assert_eq!(route.to_string_path(), "/a/b?x=1&y=2#frag");
    }

    #[test]
    fn new_normalizes_leading_slash() {
        assert_eq!(Route::new("a/b").path(), "/a/b");
        assert_eq!(Route::new("/a/b").path(), "/a/b");
        assert!(Route::new("a/b").query().is_empty());
        assert_eq!(Route::new("a/b").fragment(), None);
    }

    #[test]
    fn parse_normalizes_path() {
        assert_eq!(Route::parse("a/b").path(), "/a/b");
    }

    #[test]
    fn parse_without_query_or_fragment() {
        let route = Route::parse("/users");
        assert_eq!(route.path(), "/users");
        assert!(route.query().is_empty());
        assert_eq!(route.fragment(), None);
        assert_eq!(route.to_string_path(), "/users");
    }

    #[test]
    fn parse_key_without_value() {
        let route = Route::parse("/p?flag");
        assert_eq!(route.query_param("flag"), Some(""));
    }

    #[test]
    fn fragment_before_query_is_split_first() {
        // '?' inside the fragment stays part of the fragment.
        let route = Route::parse("/p#frag?notquery");
        assert_eq!(route.path(), "/p");
        assert!(route.query().is_empty());
        assert_eq!(route.fragment(), Some("frag?notquery"));
    }

    #[test]
    fn segments_skip_empty() {
        assert_eq!(Route::parse("/a/b/").segments(), vec!["a", "b"]);
        assert!(Route::parse("/").segments().is_empty());
    }
}
