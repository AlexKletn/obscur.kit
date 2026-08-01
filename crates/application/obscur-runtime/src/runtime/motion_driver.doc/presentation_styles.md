Layers the group-compositing presentation overlay (`opacity`/`transform`;
see [`presentation_overlays`](Self::presentation_overlays)) onto a copy of
`computed` for the scene composer, or returns [`None`] when no such motion
is active so compose reads `computed` untouched (byte-for-byte identical to
before any presentation overlay existed).

The copy is deliberate: these values reach paint but must **never** be
observable through [`computed_style`](Self::computed_style) (spec §4.17,
NG-MOTION-01). An overlaid property is written onto the node's style entry,
inserting a default entry if the node has none — so motion presents even a
node with no matched CSS at all (NG-MOTION-02).
