Where the §5.2 explicit fallback diagnostics go, once a host installs one with
[`set_diagnostic_sink`](VelloRenderer::set_diagnostic_sink); `None` until then.

`None` is not a null sink but a switch: the two display-list scans that feed
these diagnostics run only when this is `Some`. Nothing else reads their
product — `EffectFallbackReporter`'s own state is inspected solely by the CPU
backend's `Debug` — so with no sink installed they are pure per-frame cost with
no observable effect anywhere, and are skipped. Installing a sink restores the
scans, and every diagnostic a scene earns from that frame on is raised exactly
as before.
