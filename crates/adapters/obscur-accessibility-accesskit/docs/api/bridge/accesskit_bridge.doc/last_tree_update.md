Returns the latest converted [`TreeUpdate`], which the host feeds to its
platform AccessKit adapter, or [`None`] if none has been produced yet.

A push whose snapshot had no root produces no update (the conversion's
[`MissingRoot`](crate::error::AccessKitBridgeError::MissingRoot) is
swallowed here); the previous update, if any, is retained.
