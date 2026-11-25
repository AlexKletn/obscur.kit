Resolves a [`PendingMotionRule`]'s `animation` references against the
stylesheet's compiled `@keyframes`, producing a [`CompiledMotionRule`], or
[`None`] when the rule ends up carrying no drivable motion.

Each [`AnimationRef`](super::AnimationRef) is matched by interned name to a [`CompiledKeyframes`]
(the names share one interner, so equal names intern equal); an unknown name
resolves to nothing. A rule with neither transitions nor a resolvable
animation is dropped.
