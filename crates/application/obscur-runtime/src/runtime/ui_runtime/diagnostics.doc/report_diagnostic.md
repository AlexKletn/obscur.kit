Reports a `diagnostic` on the runtime's structured-diagnostics channel
(spec §1.6), through the sink installed by
[`set_diagnostic_sink`](Self::set_diagnostic_sink).

The runtime raises its own failures (component render, resource load) via
this same channel; this entry point lets a host raise a failure the runtime
cannot observe by itself but that belongs on the one channel — chiefly the
CSS frontend's parse/compile diagnostics, which are produced host-side
(from a sheet's *source*, which the runtime never receives) and would
otherwise have no way onto the channel.
