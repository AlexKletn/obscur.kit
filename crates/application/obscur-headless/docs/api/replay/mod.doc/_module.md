Record & replay subsystem (spec §7.13).

Captures a timestamped [`EventRecording`] of normalized platform events,
clock ticks, async completions, scheduler decisions, resource versions, and
window/environment changes, and replays it deterministically against test
infrastructure via [`ReplaySession`] / [`ReplayTarget`].

Sensitive input (secrets, protected clipboard data) is **redacted by
default** and irreversibly: a redacted input's payload is dropped at record
time and stored only as a marker, so recordings never carry secrets.
