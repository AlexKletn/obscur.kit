Spawns `future` onto the Tokio runtime.

[`Handle::spawn`] accepts any `Future<Output = ()> + Send + 'static`, which
[`BoxFuture`] satisfies. The returned `JoinHandle` is dropped: the port is
fire-and-forget, so the task runs detached and its completion is not
awaited here.
