The host's CSS path: owns the §4.12 parse cache across reloads, compiles each
source, and installs both halves of the result — the matchable rules **and**
the CSS-authored motion — on a runtime.

# Lifetime

One pipeline per runtime, kept alive for the app's lifetime: the cache it owns
is what makes a reload cheap, and dropping it between reloads throws that away.

# When to install

On a (re)compile — startup, and each time a source's text changes — not per
frame. [`install`](Self::install) replaces the runtime's motion rules, which
resets the per-node change-detection baselines transitions observe, so calling
it every frame would leave every transition permanently at its first frame.
