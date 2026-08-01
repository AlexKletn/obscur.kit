The typed outcome of executing a [`ClipboardRequest`](super::ClipboardRequest)
(spec §6.20 data transfer — clipboard).

Every clipboard operation reports its result as one of these honest values
rather than panicking or returning a bare `Option`. A successful paste yields
[`Read`](ClipboardResult::Read) carrying the [`DataOffer`]; a successful store
yields [`Written`](ClipboardResult::Written); a successful clear yields
[`Cleared`](ClipboardResult::Cleared). The remaining variants record honest
non-success: [`Empty`](ClipboardResult::Empty) (nothing to read),
[`Denied`](ClipboardResult::Denied) (a §6.20 permission check refused access),
and [`Unsupported`](ClipboardResult::Unsupported) (no requested format was
available or the operation is not supported). Use
[`into_error`](ClipboardResult::into_error) to fold the non-success variants
into a [`TransferError`].
