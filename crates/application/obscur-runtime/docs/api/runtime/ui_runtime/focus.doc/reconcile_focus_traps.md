Opens and closes the focus traps the committed tree declares.

A trap has no call to hang its lifetime on: an element carrying
[`FOCUS_TRAP`](obscur_interaction::focus::FOCUS_TRAP) traps focus for exactly as
long as it is committed. That is the design, not a shortcut — a dialog that
opened its trap in one place and closed it in another is a pairing an author has
to keep balanced, and the balance is lost the first time the dialog is removed by
a re-render rather than by its own close button.

So this compares the traps the tree declares now against the ones it declared
last frame:

* a trap that has **gone** closes its scope, and focus returns to whatever the
  trap displaced when it opened — the whole of §2.5's "restore focus", and the
  reason the saved frame is pushed even when it saved nothing;
* a trap that has **appeared** saves the focus it is about to take away and moves
  focus to the first tab stop inside itself;
* a trap that is **still there** has its scope re-opened, because its membership
  is the subtree and the subtree can have changed under it.

Closes run innermost-first and opens outermost-first, so nested traps unwind in
the order they were built and the innermost declared trap is the last scope on
the stack — which is the one [`focus_trap`](UiRuntime::focus_trap) answers with.
