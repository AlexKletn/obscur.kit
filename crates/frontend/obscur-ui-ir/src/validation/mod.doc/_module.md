Structural validation of UI IR documents: the [`UiIrValidator`] walks a
document and accumulates [`UiIrValidationError`]s into a
[`UiIrValidationResult`], threading a [`ValidationContext`] for cross-node
state (spec §2.5 validation).
