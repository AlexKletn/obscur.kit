The exported contract for one capability a component depends on
(spec §2.13).

A component declares the capabilities it needs so the runtime can decide
whether to instantiate it. Each dependency names a capability by its stable
[`CapabilityId`] and records whether it is [`CapabilityRequirement::Required`]
or merely [`CapabilityRequirement::Optional`].
