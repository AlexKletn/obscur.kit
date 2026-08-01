Runs one vector load to completion: `source`'s bytes parsed into a
[`SceneVector`] (spec §5.17). The vector counterpart of
[`load_image`](Self::load_image).

There is no DPI ladder here, and there is nothing missing: a vector
document is resolution-independent, so a density-suffixed variant of one
would name a file no author has a reason to write.
