Installs the BCP-47 `language` hint carried by every shaping request the
paragraph orchestrator issues (spec §7.2 text shaping integration).

This is the seam the runtime's
[`set_locale`](crate::runtime::UiRuntime::set_locale) drives with the
locale's language subtag. It reaches both the measure and the compose pass
because both shape through this one orchestrator.
