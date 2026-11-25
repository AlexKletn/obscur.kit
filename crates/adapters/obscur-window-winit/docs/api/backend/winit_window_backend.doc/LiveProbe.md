A host handler driving the whole window lifecycle through the port.

It is also the smallest demonstration of the composition a real host
performs: [`WinitAppHandler`] is itself a whole [`ApplicationHandler`], so
a host that wants its own callbacks *wraps* it and forwards, rather than
replacing it — and holds the backend alongside, the two sharing state.
