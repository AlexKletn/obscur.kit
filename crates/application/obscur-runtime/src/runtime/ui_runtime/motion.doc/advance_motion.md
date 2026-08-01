Advances every active timeline by `delta` logical seconds and, while any
motion remains, requests an [`FrameReason::Animation`] frame so the
animation keeps reaching paint (spec §4.17 "active animations keep
requesting frames").

With no animations this is a no-op: the scheduler is empty, nothing
advances, and no frame is requested. A finished timeline is pruned from the
scheduler by [`MotionScheduler::advance`]; its binding is retained so the
overlay keeps applying the settled endpoint value (the animation *holds*
its final value) until superseded.

[`MotionScheduler::advance`]: obscur_motion::scheduler::MotionScheduler::advance
