Decides, for one parent, which existing retained children a desired child
sequence should reuse, create, and remove.

This is the identity-preservation heart of reconciliation (spec §2.2,
§2.11). It is a **pure** decision function: it only reads node `kind`/`key`
through a borrowed [`RetainedTree`] and never mutates the tree. The
[`Reconciler`](super::Reconciler) applies the resulting [`ChildMatching`].

# Matching policy

A desired child may only *reuse* an existing child that is **compatible**:
the same node variant, and — for elements — the same
[`ComponentId`](obscur_identity::ComponentId). A text child reuses a text
child. An incompatible pairing is never an in-place kind change: the old
node is removed and a new one is created (a replace).

Matching proceeds in two independent lanes:

* **Keyed** desired children (those carrying a [`ViewKey`]) match the
  existing child with the *equal* [`StableKey`], regardless of position.
  This is what preserves identity across a **reorder**. A key index is built
  first (first existing occurrence of a duplicate key wins, deterministically).
  If the keyed match is incompatible, or the key is unmatched, or already
  claimed, the desired child is created.
* **Unkeyed** desired children match, scanning forward and never revisiting,
  the next unused **compatible** unkeyed existing child; otherwise they are
  created. Keyed existing children are reserved for key matching and are
  never consumed by an unkeyed desired child.

Any existing child left unmatched after both lanes is reported for removal.
The whole procedure is deterministic and panic-free; stale existing ids that
no longer resolve in the tree are ignored.
