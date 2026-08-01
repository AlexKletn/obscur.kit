Requests inhibition of system sleep / screensaver per `request`
(spec §6.5).

Reports the honest [`WindowOperationResult`]; a platform that cannot
inhibit sleep returns [`WindowOperationResult::Unsupported`].
