The capability-checked grant that a trusted rendering extension has been
permitted privileged backend access (spec §5.3).

Spec §5.3 requires that trusted, privileged extensions pass **capability
checks** before touching backend-specific GPU resources or restricted user
shaders. This context records what the host actually granted: the set of
capability ids, and whether custom shaders and raw GPU resources are allowed.
A default context grants nothing and is therefore untrusted; an extension
consults it via [`Self::has_capability`] and the allow flags before running.
