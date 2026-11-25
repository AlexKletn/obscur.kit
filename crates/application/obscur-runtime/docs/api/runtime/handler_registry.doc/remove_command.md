Unbinds the window-level handler for `command`, returning whether one was
bound.

Test-only: no shipped surface unregisters a window handler, and adding a
public one to serve a test is how a method with no production caller gets
into a codebase. The §2.6 chain test needs it to strip one link at a time
and watch the next claim the command.
