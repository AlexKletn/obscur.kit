#![doc = include_str!("navigation_result.doc/_module.md")]

use crate::route::RouteParams;

#[doc = include_str!("navigation_result.doc/NavigationResult.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationResult {
    #[doc = include_str!("navigation_result.doc/Matched.md")]
    Matched {
        #[doc = include_str!("navigation_result.doc/target.md")]
        target: Box<str>,
        #[doc = include_str!("navigation_result.doc/params.md")]
        params: RouteParams,
        #[doc = include_str!("navigation_result.doc/path.md")]
        path: Box<str>,
    },
    #[doc = include_str!("navigation_result.doc/NotFound.md")]
    NotFound {
        #[doc = include_str!("navigation_result.doc/path_2.md")]
        path: Box<str>,
    },
}

impl NavigationResult {
    #[doc = include_str!("navigation_result.doc/matched.md")]
    #[must_use]
    pub fn matched(
        target: impl Into<Box<str>>,
        params: RouteParams,
        path: impl Into<Box<str>>,
    ) -> Self {
        Self::Matched {
            target: target.into(),
            params,
            path: path.into(),
        }
    }

    #[doc = include_str!("navigation_result.doc/not_found.md")]
    #[must_use]
    pub fn not_found(path: impl Into<Box<str>>) -> Self {
        Self::NotFound { path: path.into() }
    }

    #[doc = include_str!("navigation_result.doc/is_matched.md")]
    #[must_use]
    pub const fn is_matched(&self) -> bool {
        matches!(self, Self::Matched { .. })
    }

    #[doc = include_str!("navigation_result.doc/target_2.md")]
    #[must_use]
    pub fn target(&self) -> Option<&str> {
        match self {
            Self::Matched { target, .. } => Some(target),
            Self::NotFound { .. } => None,
        }
    }

    #[doc = include_str!("navigation_result.doc/params_2.md")]
    #[must_use]
    pub const fn params(&self) -> Option<&RouteParams> {
        match self {
            Self::Matched { params, .. } => Some(params),
            Self::NotFound { .. } => None,
        }
    }

    #[doc = include_str!("navigation_result.doc/path_3.md")]
    #[must_use]
    pub fn path(&self) -> &str {
        match self {
            Self::Matched { path, .. } | Self::NotFound { path } => path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matched_helpers() {
        let mut params = RouteParams::new();
        params.insert("id", "42");
        let result = NavigationResult::matched("user", params, "/users/42");
        assert!(result.is_matched());
        assert_eq!(result.target(), Some("user"));
        assert_eq!(result.params().and_then(|p| p.get("id")), Some("42"));
        assert_eq!(result.path(), "/users/42");
    }

    #[test]
    fn not_found_helpers() {
        let result = NavigationResult::not_found("/missing");
        assert!(!result.is_matched());
        assert_eq!(result.target(), None);
        assert!(result.params().is_none());
        assert_eq!(result.path(), "/missing");
    }
}
