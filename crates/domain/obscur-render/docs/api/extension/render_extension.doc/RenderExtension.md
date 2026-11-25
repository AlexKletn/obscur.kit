A privileged rendering extension bundling custom passes behind a required
capability (spec §5.3).

Trusted extensions may inject backend-specific [`CustomPass`]es, but only
after the host's capability check passes (spec §5.3). This value pairs the
passes with the [`Self::required_capability`] the host must have granted and
evaluates that check via [`Self::is_permitted`]. It never redefines ordinary
Canvas semantics — it only schedules additional privileged work.
