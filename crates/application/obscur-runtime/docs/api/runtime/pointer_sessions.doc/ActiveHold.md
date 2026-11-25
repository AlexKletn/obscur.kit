A long press armed for one pointer: the recognizer counting the hold, the
node the contact went down on, where it went down, and whether the platform
vouched for the contact.

The trust is carried from the `Down` because a long press *is* a §2.3 default
action, gated exactly like a click's: an injected contact may be observed,
but it must not run the handler a real finger would.
