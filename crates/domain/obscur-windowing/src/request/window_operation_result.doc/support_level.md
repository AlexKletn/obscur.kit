Returns the support level of this outcome, if it maps to one.

[`WindowOperationResult::Performed`] returns its recorded level;
[`WindowOperationResult::Unsupported`] returns
[`SupportLevel::Unavailable`]; a runtime
[`WindowOperationResult::Failed`] returns `None`, as failure is not a
support category.
