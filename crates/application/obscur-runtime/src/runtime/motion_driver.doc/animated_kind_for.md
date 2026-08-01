Returns how a scalar animates into `property`'s computed value, or [`None`]
for a property the motion overlay cannot express as a scalar.

This mirrors [`AnimationBinding`]'s lowering: `opacity`/`flex-grow`/
`flex-shrink` are bare numbers; box-model lengths are pixel lengths. A
property outside this set (e.g. a colour or keyword) is not scalar-animatable
here and drives no CSS transition/animation.
