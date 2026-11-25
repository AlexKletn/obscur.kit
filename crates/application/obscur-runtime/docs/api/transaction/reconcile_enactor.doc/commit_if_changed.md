Closes a validated, already-applied reconcile atomically: only a plan that
changed the tree advances the [`version`](Self::version) and carries the
staged operations; an idempotent pass commits empty and leaves the stamp
untouched.

Shared by [`commit_root`](Self::commit_root) and
[`commit_subtree`](Self::commit_subtree) so the two enact on identical
version terms.
