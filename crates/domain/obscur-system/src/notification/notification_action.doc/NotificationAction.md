An actionable button on a [`Notification`](crate::notification::Notification)
(spec §6.12 — notification actions).

An action is a pure value: a stable [`id`](Self::id) the host echoes back when
the user clicks the button, paired with the [`label`](Self::label) shown on it.
The domain matches the echoed id against the actions it attached; the display
text is never used for identity.
