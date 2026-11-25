Runs `job` on smol's blocking thread pool, off the caller's thread.

**This does not touch [`Self::executor`], and that is the point.** The
[`Spawner`] half of this type drives its executor with exactly *one*
worker thread, shared with every [`SmolTimer`](crate::timer::SmolTimer)
built from [`SmolSpawner::executor`]. A blocking job — the whole point of
this port is that it blocks — would own that thread for its duration and
stall every future and timer queued behind it. So blocking work goes to
[`blocking::unblock`]'s dedicated, dynamically-grown pool instead, which
is the same split smol itself draws between `smol::spawn` and
`smol::unblock`. The two halves therefore share nothing but the type.

The returned `Task` is detached: this port is fire-and-forget, and a
detached task still runs — `unblock` schedules its runnable onto the pool
before returning it. A job that panics does not take a pool thread with
it: `blocking` builds its tasks with `propagate_panic(true)`
(`blocking-1.6.2/src/lib.rs:219`), which catches the unwind inside
`Runnable::run` and re-raises it only when the `Task` is polled — and a
detached task is never polled. The result simply never arrives, which for
a resource load means the image is not painted.

# Panics

On `wasm` targets `blocking`'s pool cannot exist and `unblock` panics
("cannot spawn a blocking task on WASM", `blocking-1.6.2/src/lib.rs:212`).
§1.5's WASM workers are a separate, unbuilt facility; do not install this
adapter on a WASM host.
