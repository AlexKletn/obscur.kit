The pending and lifetime-serviced tallies of **background task work** (spec
§3.6 band 5: background shaping, decoding and cache work).

# Why these two are one entity

[`service`](Self::service) is the only operation that writes either, and it
must write **both**: a unit leaves the pending queue exactly when it enters
the serviced total. Written apart, the two tallies are an accounting that can
silently stop adding up — a decrement that forgets the total claims work
vanished, an increment that forgets the queue claims work that never ran.

The second half of the invariant is the **edge**. The scheduler is told
background work is waiting on the `0 → 1` transition and idle again on
`1 → 0`, and those two conditions used to be written as `== 1` and `== 0`
guards in two different methods. Getting the second one wrong is the
expensive direction and it is silent: a scheduler still counting preemptions
for a drained queue reports starvation forever, and the drain loop promotes a
background unit that is not there on every frame it runs.

# What it deliberately does not do

It does not hold the [`FrameScheduler`](crate::scheduler::FrameScheduler).
Both operations **report** their edge and the runtime signals the scheduler —
the same choice `SurfaceGeometry` makes when a scale change crosses an asset
band. The tallies are then a fact about the queue alone, and the fairness
state stays where fairness is decided.
