The top-level `.okui` compilation orchestrator: sequences resolution,
validation and lowering over one parsed document into UI IR (spec §2.9–§2.15).

The individual passes ([`NameResolver`](crate::resolution::NameResolver),
the validators in [`crate::validation`], [`UiIrLowering`]) remain usable
standalone; [`Compiler`] merely runs them in order, accumulating every
pass's diagnostics into one list rather than bailing on the first failure.
