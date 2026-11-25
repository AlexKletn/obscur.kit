Samples motion tracks into presentation values: [`MotionCoordinator`].

This is the runtime-side orchestration entry point for the motion engine
(spec §4.17). The timeline integration, easing curves and value
interpolation all live in `obscur-motion`; the coordinator's job is to turn a
track plus a normalized progress into a concrete [`MotionValue`] the runtime
can overlay onto a node's computed style before layout and paint.
