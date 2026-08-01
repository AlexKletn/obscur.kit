The router: a [`RouteRegistry`] composed with a history provider (spec
§6.13).

The router is the toolkit's composition point for URL-like navigation. It
owns the pattern-to-target [`RouteRegistry`] and a boxed
[`HistoryProvider`], and turns a path into a typed [`NavigationResult`] while
maintaining the back/forward history stack. Application navigation *policy*
(guards, redirects, what a target means) stays outside the toolkit.

`Router` derives only [`Debug`] — not `Clone` or `PartialEq` — because it
holds a `Box<dyn HistoryProvider>`, a trait object that is neither `Clone`
nor `PartialEq`. The [`HistoryProvider`] trait carries a
[`std::fmt::Debug`] supertrait so the boxed provider is `Debug`.
