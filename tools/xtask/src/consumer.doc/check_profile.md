Build and test one fixture, then read what the build said.

A failure is reported with the compiler's own output. A success is not the end
of it: the build log is still scanned for warnings in the profile's
[`watched`](ConsumerProfile::watched) paths, because a warning nobody runs the
build to see is how a defect stays open.

Builds into `target/checks/<rule>` rather than beside the fixture, so the one
directory CI caches covers these too.
