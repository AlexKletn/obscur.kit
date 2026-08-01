A queryable platform capability key for [`WindowBackend::supports`].

Each variant names one capability the spec (§6.2-6.6) flags as
platform-conditional, so an application can introspect the honest
[`SupportLevel`] a backend offers before relying on it.
