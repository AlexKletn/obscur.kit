The failure type for the fallible window-creation path
([`WindowBackend::create_window`], spec §6.1).

Creation is the one operation modeled as `Result` rather than a
[`WindowOperationResult`], because a failure yields no [`WindowId`] to
operate on. Non-creation operations report their honest outcome through
their own typed result types instead.
