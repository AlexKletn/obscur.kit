The matchable sheets from the last [`install`](Self::install), for the
render paths that take an explicit `&[Stylesheet]`
([`render_now`](crate::runtime::UiRuntime::render_now) and friends). The
reactive path reads the copy `install` left on the runtime instead.
