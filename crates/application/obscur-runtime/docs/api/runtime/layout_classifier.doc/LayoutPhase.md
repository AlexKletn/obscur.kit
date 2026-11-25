What this frame's layout phase must do (spec §3.4 step 8 "perform incremental
layout", §4.3).

The three answers [`LayoutClassifier::classify`] can give, in increasing cost.
This is the layout twin of the style phase's
[`RestyleScope`](crate::invalidation::RestyleScope)/[`StyleInvalidation`]
pair: the classifier does not decide *whether* to run a phase and then run all
of it — it decides *what* the phase has to touch.
