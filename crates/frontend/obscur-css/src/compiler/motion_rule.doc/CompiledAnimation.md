A resolved `animation` on a rule: the `@keyframes` tracks to play, the
duration to play them over, and the playback controls the `animation-*`
longhands set.

Produced by resolving an `animation-name` reference against the stylesheet's
compiled `@keyframes`; an unknown name contributes no [`CompiledAnimation`].
