A single normalized UI operation: a typed intent that the frame pipeline
interprets against the domains (tree, style, layout).

Operations are *normalized*: a mutation coming from application code is
reduced to one of these variants before it enters a [`UiTransaction`], so
the runtime never dispatches on strings and always reasons over a closed set
of intents. This is only the *intent* — nothing is applied when a
`UiOperation` is created. The concrete effect happens later in the frame
pipeline once the transaction has been validated and committed.

# What an operation actually does

[`invalidation`](Self::invalidation) is where an intent becomes an effect: it
states the change as an [`Invalidation`], which the
[`InvalidationRouter`](crate::invalidation::InvalidationRouter) turns into the
set of pipeline phases a frame must run. That is the whole of it — an
operation *classifies work*, it does not describe an edit. The edit itself
comes from reconciling the desired [`View`](obscur_view::view::View); an
operation says which phases have to look at the result.

This is the Phase-1 set; more operation kinds are added in later phases.

[`UiTransaction`]: crate::transaction::UiTransaction
