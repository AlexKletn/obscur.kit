Returns this event marked as a real user gesture (fluent).

**Only a platform adapter converting an actual OS input event may call
this.** It is the single point at which trust enters the runtime; every
other producer (DevTools `Inject` per §7.6, the §7.4 headless harness)
must leave the event untrusted.
