Activates a window and optionally moves element focus per `request`
(spec §6.3).

Returns [`ActivationResult`]: focus-stealing prevention is a normal
[`ActivationResult::Denied`] outcome, and a backend without programmatic
activation returns [`ActivationResult::Unsupported`] — neither is an
error.
