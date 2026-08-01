The **endpoint** end of the injection channel: where an authorized `Inject`
is delivered (spec §7.6).

Obtained from [`InjectionQueue::sink`] and installed with
[`DevToolsEndpoint::with_injection_sink`](crate::endpoint::DevToolsEndpoint::with_injection_sink).
It holds a [`Weak`], so it reports [`is_live`](Self::is_live) only while the
host's queue still exists — an endpoint outliving its host's drain end refuses
injection rather than filling a queue nobody will ever read.
