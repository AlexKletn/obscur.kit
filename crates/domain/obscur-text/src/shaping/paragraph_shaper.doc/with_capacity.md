Creates a shaper whose paragraph cache holds at most `capacity` results
across its two generations (a `capacity` below two is raised to two, so a
generation always has room to keep something).

Reaching the bound does **not** empty the cache: it drops the older
generation only, retaining everything inserted or looked at since the
previous rotation. The wholesale flush it replaces was catastrophic for a
working set larger than the bound — every insertion past it emptied the
table, so a long list re-shaped every visible run on every frame, forever.
