The stateless helper that maps a hit chain to a logical propagation path
(spec §4.7 hit testing → event routing).

Given the ancestor chain from the root down to a hit-test target,
[`EventDispatcher`] produces the ordered capture → target → bubble sequence an
event travels, and can [`dispatch`](Self::dispatch) an event along it. The
dispatcher carries no state and reads no clock: it is a pure, deterministic
function of its inputs. Running the handlers attached to each node is the
runtime's concern, not this crate's — here [`dispatch`](Self::dispatch) only
computes and validates the path and honors any pre-set propagation control.
