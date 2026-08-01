Navigates to `path`, attaching `state` on a match (spec §6.13).

The `path` is parsed into a [`Route`] and resolved against the registry.
On a match the route is pushed onto history (carrying `state`) and a
[`NavigationResult::Matched`] is returned. On no match a
[`NavigationResult::NotFound`] is returned and **history is not mutated** —
a failed navigation leaves the back/forward stack untouched.
