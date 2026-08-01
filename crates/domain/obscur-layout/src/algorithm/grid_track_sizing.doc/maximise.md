§11.6 — *Maximise Tracks*. Distributes the free space (`avail − Σ base`) to
every track's base up to its growth limit. (The spec's "infinite free space"
branch — for an indefinite container — is subsumed here: with a finite but
ample `avail`, every track reaches its growth limit anyway, and `auto`/`fr`
tracks have `limit == base` so they are untouched.)
