Reports the honest [`SupportLevel`] at which this backend offers
`feature` (spec §6.2-6.6).

Returns [`SupportLevel::Unavailable`] for a feature the platform does not
provide; this is the introspection counterpart of the per-operation
typed results.
