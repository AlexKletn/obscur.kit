Returns this event marked as a real user gesture (fluent).

**Only the layer that received the OS input may call this**, for the
reason spelled out on [`EventTrust`]: trust is conferred at the platform
boundary and never assumed, so a forgotten call costs an action while a
careless one forges a gesture.
