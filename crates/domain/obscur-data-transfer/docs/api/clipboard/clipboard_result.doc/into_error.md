Folds a non-success outcome into a [`TransferError`], returning `None`
for any success variant (spec §6.20).

The mapping is: [`Empty`](ClipboardResult::Empty) →
[`TransferError::Empty`], [`Denied`](ClipboardResult::Denied) →
[`TransferError::PermissionDenied`],
[`Unsupported`](ClipboardResult::Unsupported) →
[`TransferError::Unsupported`], and each success variant
([`Read`](ClipboardResult::Read), [`Written`](ClipboardResult::Written),
[`Cleared`](ClipboardResult::Cleared)) → `None`.
