//! A compiled route pattern with static, parameter, and wildcard segments
//! (spec §6.13).

use crate::route::RouteParams;

/// A single segment of a [`RoutePattern`] (spec §6.13).
///
/// Patterns are compiled into a sequence of these segments, one per non-empty
/// path component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternSegment {
    /// A literal segment that must equal the corresponding path segment.
    Static(Box<str>),
    /// A `:name` segment that captures exactly one path segment under `name`.
    Param(Box<str>),
    /// A `*name` segment that captures all remaining path segments (joined by
    /// `/`) under `name` and ends the match.
    Wildcard(Box<str>),
}

/// A compiled route pattern such as `/users/:id/posts/:postId` or
/// `/files/*rest` (spec §6.13).
///
/// A pattern is a sequence of [`PatternSegment`]s matched against a path's
/// segments by [`match_path`](RoutePattern::match_path). Nested routes are
/// expressed simply as longer patterns (deeper paths) or a trailing
/// [`Wildcard`](PatternSegment::Wildcard) that absorbs an arbitrarily deep
/// tail; the toolkit does not maintain a separate route tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutePattern {
    /// The compiled segments, in path order.
    segments: Vec<PatternSegment>,
}

impl RoutePattern {
    /// Compiles a pattern string into a [`RoutePattern`] (spec §6.13).
    ///
    /// The string is split on `/` and empty components are skipped, so leading
    /// and trailing slashes are irrelevant. Each remaining component becomes a
    /// segment: `:name` is a [`Param`](PatternSegment::Param), `*name` is a
    /// [`Wildcard`](PatternSegment::Wildcard), and anything else is a
    /// [`Static`](PatternSegment::Static) literal. A pattern with no segments
    /// (for example `/`) matches only the root path.
    #[must_use]
    pub fn parse(pattern: &str) -> Self {
        let segments = pattern
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| {
                if let Some(name) = s.strip_prefix(':') {
                    PatternSegment::Param(Box::from(name))
                } else if let Some(name) = s.strip_prefix('*') {
                    PatternSegment::Wildcard(Box::from(name))
                } else {
                    PatternSegment::Static(Box::from(s))
                }
            })
            .collect();
        Self { segments }
    }

    /// Returns the compiled pattern segments.
    #[must_use]
    pub fn segments(&self) -> &[PatternSegment] {
        &self.segments
    }

    /// Returns whether the pattern is fully static (no param or wildcard
    /// segments).
    #[must_use]
    pub fn is_static(&self) -> bool {
        self.segments
            .iter()
            .all(|s| matches!(s, PatternSegment::Static(_)))
    }

    /// Matches `path` against the pattern, returning the captured
    /// [`RouteParams`] on success (spec §6.13).
    ///
    /// The path is split into non-empty segments and consumed left to right
    /// against the pattern segments:
    ///
    /// - [`Static`](PatternSegment::Static) must equal the current path
    ///   segment.
    /// - [`Param`](PatternSegment::Param) captures one path segment under its
    ///   name.
    /// - [`Wildcard`](PatternSegment::Wildcard) captures *all* remaining path
    ///   segments (re-joined with `/`) under its name and ends the match; a
    ///   wildcard with no remaining segments captures the empty string.
    ///
    /// A match succeeds only when the pattern and path are consumed in step:
    /// every path segment must be accounted for (unless a wildcard absorbed the
    /// tail) and every non-wildcard pattern segment must have a path segment.
    /// Returns `None` on any mismatch. An empty pattern matches only a path
    /// with no segments (the root `/`).
    #[must_use]
    pub fn match_path(&self, path: &str) -> Option<RouteParams> {
        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut params = RouteParams::new();
        let mut index = 0;

        for segment in &self.segments {
            match segment {
                PatternSegment::Static(literal) => {
                    let actual = path_segments.get(index)?;
                    if actual != &&**literal {
                        return None;
                    }
                    index += 1;
                }
                PatternSegment::Param(name) => {
                    let actual = path_segments.get(index)?;
                    params.insert(&**name, *actual);
                    index += 1;
                }
                PatternSegment::Wildcard(name) => {
                    let rest = path_segments[index..].join("/");
                    params.insert(&**name, rest);
                    return Some(params);
                }
            }
        }

        // No wildcard consumed the tail: every path segment must be matched.
        if index == path_segments.len() {
            Some(params)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_segment_kinds() {
        let pattern = RoutePattern::parse("/users/:id/*rest");
        assert_eq!(
            pattern.segments(),
            &[
                PatternSegment::Static(Box::from("users")),
                PatternSegment::Param(Box::from("id")),
                PatternSegment::Wildcard(Box::from("rest")),
            ]
        );
        assert!(!pattern.is_static());
        assert!(RoutePattern::parse("/a/b").is_static());
    }

    #[test]
    fn match_param() {
        let pattern = RoutePattern::parse("/users/:id");
        let params = pattern.match_path("/users/42").expect("should match");
        assert_eq!(params.get("id"), Some("42"));
        assert_eq!(params.get_parsed::<u32>("id"), Some(42));
    }

    #[test]
    fn static_mismatch() {
        let pattern = RoutePattern::parse("/users/:id");
        assert!(pattern.match_path("/posts/42").is_none());
    }

    #[test]
    fn wildcard_captures_tail() {
        let pattern = RoutePattern::parse("/files/*rest");
        let params = pattern.match_path("/files/a/b/c").expect("should match");
        assert_eq!(params.get("rest"), Some("a/b/c"));
    }

    #[test]
    fn wildcard_captures_empty_tail() {
        let pattern = RoutePattern::parse("/files/*rest");
        let params = pattern.match_path("/files").expect("should match");
        assert_eq!(params.get("rest"), Some(""));
    }

    #[test]
    fn length_mismatch_returns_none() {
        let pattern = RoutePattern::parse("/users/:id");
        // Too many path segments.
        assert!(pattern.match_path("/users/42/extra").is_none());
        // Too few path segments.
        assert!(pattern.match_path("/users").is_none());
    }

    #[test]
    fn empty_pattern_matches_root() {
        let pattern = RoutePattern::parse("/");
        assert!(pattern.segments().is_empty());
        assert!(pattern.match_path("/").is_some());
        assert!(pattern.match_path("/a").is_none());
    }
}
