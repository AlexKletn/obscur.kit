Returns a copy of this track whose first (lowest-offset) keyframe holds
`start`, leaving every other keyframe — offsets, easings and later values
— untouched (spec §4.17 "interruption / blend").

This is the primitive a
[`Blend`](crate::transition::TransitionInterruption::Blend) interruption
uses: an incoming transition is re-based so it *begins* at the element's
current mid-animation value instead of the track's authored start,
producing a seamless redirect with no visible jump. An empty track is
returned unchanged.
