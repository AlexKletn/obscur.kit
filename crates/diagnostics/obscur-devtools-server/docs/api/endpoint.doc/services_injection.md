Whether an authorized `Inject` would actually reach a host — i.e. whether
a live [`InjectionSink`] is installed (see
[`with_injection_sink`](Self::with_injection_sink)).

False on a fresh endpoint and false again once the host drops its
[`InjectionQueue`](crate::transport::InjectionQueue), which is what makes
this a fact about reachability rather than a host's promise.
