The difference between two computed styles: [`StyleDiff`].

[`StyleDiff::between`] compares an `old` and a `new` [`ComputedStyle`] and
records which computed properties changed and whether any custom property
changed. Downstream invalidation (sub-step 6) consumes this to decide what to
re-layout or re-paint. The changed-property list is sorted for stable,
deterministic output.
