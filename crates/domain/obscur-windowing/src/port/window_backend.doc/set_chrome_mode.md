Sets the chrome mode (server- vs client-side decorations) for window `id`
(spec §6.4).

Reports the honest [`WindowOperationResult`]; a platform that cannot
honor the requested mode returns [`WindowOperationResult::Unsupported`].
