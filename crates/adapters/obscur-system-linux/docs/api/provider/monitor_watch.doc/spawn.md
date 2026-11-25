Start the tool and call back on every line that mentions the marker.

`None` if the tool will not start, which is the same answer as "this source
cannot be watched" and needs no separate error.

The callback is invoked from the reader thread. It is never given the line: what
changed is deliberately not reported (see
[`AppearanceCallback`](obscur_system::appearance::AppearanceCallback)), so a
caller re-reads rather than trusting a second parser.
