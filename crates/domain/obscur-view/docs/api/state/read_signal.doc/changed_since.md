Returns `true` if this snapshot is newer than the generation the consumer
last observed, i.e. the value changed since `last_seen`.

Equal generations report `false`, so unchanged sources trigger no work.
