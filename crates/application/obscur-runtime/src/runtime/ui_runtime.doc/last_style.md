The style-phase inputs the retained `inputs`/`computed` were produced from,
or `None` until the first style phase runs.

An incremental restyle reuses those retained outputs for every node it does
not re-resolve, which is sound only while everything they were computed
from is unchanged. This is that "everything": see [`StyleKey`].
