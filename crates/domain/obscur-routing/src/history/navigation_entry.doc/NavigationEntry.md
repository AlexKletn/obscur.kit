One entry in the navigation history stack (spec §6.13).

An entry pairs the [`Route`] that was navigated to with the opaque
[`RouteState`] captured for it (scroll position, form data, a selection) and
a monotonic [`id`](NavigationEntry::id) assigned by the history provider. The
id lets a caller distinguish two entries with the same route (for example the
same page visited twice) without comparing their state.
