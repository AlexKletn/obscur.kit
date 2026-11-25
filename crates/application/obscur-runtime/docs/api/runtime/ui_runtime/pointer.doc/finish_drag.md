Ends an in-flight drag on a pointer release: commits the drop's staged work
(as one transaction, through [`apply_transaction`](Self::apply_transaction))
or discards it.

The commit path is what makes the operation vocabulary do something here: a
drop's [`Rerender`](crate::transaction::UiOperation::Rerender) classifies
the frame as one that must reconcile, where a cancel classifies nothing and
applies nothing.

# It does not narrow the reconcile to the dropped-on node

An earlier version of this comment claimed it did. It does not, and the
operation cannot: [`FramePlan::for_transaction`] reduces the whole
transaction to four booleans, discarding every `node` payload, and a
reconciling frame then calls `commit_root` over the entire desired
[`View`]. The node is dropped on the floor.

Nor could it narrow without more than an operation: a `Rerender { node }`
says *that* node's content changed, but the new content comes from the
application's rebuilt view, and only the component model
([`ComponentRenderPass`], which does use
`commit_subtree`) knows how to obtain a sub-view for one node. Until a drop
routes through that, `Rerender`'s payload is a claim nothing acts on.

What commit/cancel *does* buy is real and is what the tests assert: a drop
schedules a frame that reconciles, and an abandoned drag schedules none.
