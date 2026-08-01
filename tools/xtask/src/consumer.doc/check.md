Builds and tests the fixture, reporting a violation if it does not pass.

Shells out to `cargo` rather than reimplementing anything: the claim under
test is "an application's build succeeds", and only a real build can make it.
