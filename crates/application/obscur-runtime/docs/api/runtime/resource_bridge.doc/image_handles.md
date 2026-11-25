The [`ResourceHandle`] minted for each requested image URL, retained so the
hub lifecycle (begin_load → complete/fail) is driven per resource and a URL
is requested at most once.

Kept to the URLs that are either referenced by this frame's tree/styles or
still resident in a cache — [`retain_referenced`](Self::retain_referenced)
releases the rest back to the hub. Without that it is one entry per URL the
program has EVER shown, and a file manager with per-file thumbnails walks a
directory into a permanent map.
