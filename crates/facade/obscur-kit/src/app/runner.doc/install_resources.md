Installs the filesystem image resources and the threaded blocking executor when
their adapters are compiled in (both are in the `full` set the windowed feature
gate implies). A standalone window is not a headless host, so the *real*
executor is correct here — a `url()` image lands a frame later and
[`poll_resource_tasks`](UiRuntime::poll_resource_tasks) on wake repaints for it.
