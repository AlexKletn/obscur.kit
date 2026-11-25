Asks the compositor to report when the next committed frame is shown.

Costs one protocol object per frame, which is why it is a request and not a
standing subscription: a runner that is not measuring never calls it.
