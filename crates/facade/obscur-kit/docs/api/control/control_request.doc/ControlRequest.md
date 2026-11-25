Something an application asked the runtime to do, waiting for the point in the
frame where doing it is safe.

Only the requests that need the *host's* collaborators are queued: replacing the
authored stylesheets needs the [`CssPipeline`](crate::bootstrap::CssPipeline)
that owns §4.12's parse cache, and an arbitrary mutation needs a `&mut
UiRuntime`. A positional registration is not here — it needs no runtime to be
*recorded*, only to be *resolved*, so it goes straight into
[`CommandBindings`](super::CommandBindings) and is resolved on the same pass.
