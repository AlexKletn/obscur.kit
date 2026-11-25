Closes the window named by `request`, honoring or bypassing the
close-confirmation per [`CloseWindow::is_forced`] (spec §6.1).

Reports the honest outcome via [`WindowOperationResult`]; an unknown
target yields [`WindowOperationResult::Failed`], never a panic.
