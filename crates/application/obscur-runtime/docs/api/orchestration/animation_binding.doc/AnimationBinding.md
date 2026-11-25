One active animation: the timeline driving it, the node and property it
targets, the keyframe track it samples, and how to lower the sample into a
computed value (spec §4.17 style → motion → paint).

The [`UiRuntime`](crate::UiRuntime) owns the
[`MotionScheduler`](obscur_motion::scheduler::MotionScheduler) that advances
the timeline; the binding is the registry entry that tells the runtime *what*
the timeline animates, so each frame's overlay pass can sample the track at
the timeline's progress and write the result onto the target node's computed
style before layout and paint.
