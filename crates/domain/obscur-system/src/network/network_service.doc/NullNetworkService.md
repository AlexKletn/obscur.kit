The honest kit default: a [`NetworkService`] that performs **no** network
access (spec §5.15).

`NullNetworkService` enforces the §5.15 policy that the toolkit does no
network I/O by itself: every [`fetch`](NetworkService::fetch) returns
[`NetworkError::Unsupported`] and [`availability`](NetworkService::availability)
reports [`SupportLevel::Unavailable`]. Its refusals are correct behavior,
not stubs waiting to be filled in — a host that wants network access
supplies its own adapter.
