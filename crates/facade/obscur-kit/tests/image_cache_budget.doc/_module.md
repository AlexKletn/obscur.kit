End-to-end verification that the **§5.19 cache tier runs on the live frame
path**: the runtime's decoded-image cache admits through a [`CacheBudget`],
evicts by an [`EvictionPolicy`], scores victims by [`VisibilityPriority`], and
reports [`CacheStatistics`] — driven by real PNGs on disk, the real
`FsResourceLoader`, the real `image`-crate decoder, and a real `render_now`
frame (spec §5.19 *memory budgets; eviction; visibility priority; `DevTools`
metrics*).

This closes audit finding 5-C ("§5.19 is a complete, tested, entirely unused
subsystem" — a bounded tier nothing called, next to an unbounded `HashMap`
everything called). The point of these tests is that **nothing here touches the
cache directly**: every insert and every eviction is a side effect of composing
a frame over authored CSS, which is the only evidence that the tier is on the
production path rather than merely unit-tested.
