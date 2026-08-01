Returns the runtime this host drives, for configuration and driving the
host does not wrap — registering motion ([`UiRuntime::animate`],
[`UiRuntime::set_css_motion`]), installing resource providers, or queueing
synthetic input.

The clock and executor stay in charge of time and background work: the
executor is already installed as the runtime's blocking port, and motion
registered here is advanced by [`advance_time`](Self::advance_time).
Calling [`UiRuntime::advance_motion`] through this handle bypasses the
clock and forfeits that guarantee.
