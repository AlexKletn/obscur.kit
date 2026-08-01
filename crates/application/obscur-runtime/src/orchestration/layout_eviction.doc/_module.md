What a [`LayoutInvalidation`] can have changed: [`invalidated_layout`].

The §4.3 dependency analysis that stands between the runtime's dirty set and
the two retained caches. Its interface is one call answering one question —
"which retained entries must go?" — and its internals are the written-down
dependency list of `measure(N, c)` plus the two tree walks that close it
upward. Neither cache can perform this analysis itself: it is a fact about
the *tree*, and both caches are keyed by node alone.
