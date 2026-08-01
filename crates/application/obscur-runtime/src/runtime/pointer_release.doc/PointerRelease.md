What one release (an `Up`) resolved to against the press it ends: whether it
was a **click**, on what, with what run length, and whether an outstanding
press went out with it (spec §2.3, §5.11).

Taking this *consumes* the press: the anchor is cleared on every release,
clicked or not.

# Why the three conditions are one decision

A release is a click only if all three hold, and each of them was, at some
point, the bug:

- **Its own press.** Not whichever press happened to be the most recent. A
  second finger landing mid-gesture used to overwrite the single anchor, so
  the first finger's release found the wrong target and activated nothing.
- **The same node.** A press that slid off the control it started on is not a
  click on it — that is the whole difference between pressing a button and
  changing your mind.
- **Trusted.** §2.3 propagation step 4, the **default action**, is gated on
  trust. Activation — resolving the node's bound command and running its
  handler — is the gated default action, and it runs only for a
  platform-vouched gesture. A synthetic event (a headless dispatch §7.4, a
  DevTools inject §7.6) still dispatches and is observed (it moved hover and
  focus), but its gated default action is suppressed: nothing here reports a
  click, so no command resolves and no handler fires. This is the host control
  §9 forbids weakening and the trust boundary §1.8 names.

Written as one conjunction in the middle of the dispatch, any of the three can
be dropped by a later edit with nothing to notice: two of them fail *silently*
— the control simply does not respond — and the third fails by responding to
input the platform never vouched for.
