An evicted image is not lost forever: a later frame that still references it
loads it again. Eviction has to release the hub bookkeeping to make that true —
the "already requested, do not retry" guard would otherwise silently retire the
URL for the life of the process.
