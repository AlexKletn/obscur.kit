Records that the value behind `source` changed, and marks every component
that depends on it stale (spec §3.1: "dependency tracking invalidates only
consumers of changed values").

Returns how many components were invalidated — zero when nothing reads the
source, which is the normal and cheap case for an app with many sources.

A component that tracked a *different* source is untouched, and one that
tracked nothing is never invalidated by any notification. That asymmetry is
the whole point: it is what a wholesale root rebuild cannot express.
