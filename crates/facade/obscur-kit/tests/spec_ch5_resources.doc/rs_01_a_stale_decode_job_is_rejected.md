§5.15 "Features: … - versioning; - stale-job rejection; - development hot
reload;".

A job that started under version 1 and finishes *after* a hot reload to
version 2 must be discarded — last-writer-wins is correct in almost every
ordering, which is exactly why the inverted order has to be driven explicitly.
