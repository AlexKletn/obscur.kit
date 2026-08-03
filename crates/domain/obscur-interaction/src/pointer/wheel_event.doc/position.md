Where the pointer was, in the same space a dispatched pointer event's position
is in — content space, folded through the scroll containers the point lies
within. That is the space the hit test that found the target ran in, so a handler
comparing this against a node's box is comparing like with like.
