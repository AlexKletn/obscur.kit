The pending and lifetime-serviced tallies of background task work (spec
§3.6 band 5: background shaping / decode / cache work).

Background work is a **signal, not a frame**: it is separate subsystem
task work driven by budgeting and starvation prevention rather than by a
[`FrameReason`]. A host enqueues a unit via
[`request_background`](Self::request_background); the scheduler is told
work is waiting ([`FrameScheduler::note_background_pending`]), and every
higher-priority frame drained afterwards counts as a preemption. When the
scheduler reports the work starved
([`FrameScheduler::background_starved`]) the drain loop services one unit
on the live path, honouring §3.6 "starvation prevention for background
work".

One field rather than the two counters it replaces: they are one
accounting, and the scheduler signal above is edge-triggered off it — see
[`BackgroundWork`].
