What can go wrong between asking for a layer surface and having one.

Connection, registry initialization, roundtrip, flush, and event-loop failures
expose their underlying library or OS error through `std::error::Error::source`.
Invalid settings describe the rejected request.
