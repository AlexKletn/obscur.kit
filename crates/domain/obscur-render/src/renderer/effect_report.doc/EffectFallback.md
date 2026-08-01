One fallback a backend actually took while drawing a scene: the effect that was
degraded, and the backend's own account of what it drew instead (spec §5.2).

The effect is drawn from the closed [`SceneEffect`] vocabulary so a host can
route on it; the reason is free text because *why* an approximation bit is
backend-specific in a way the port cannot enumerate (an unregistered font handle,
an unpeelable nested filter layer). Both together are the identity: two runs
missing two different fonts are two distinct fallbacks and are reported
separately, while the same run missing the same font every frame is one.
