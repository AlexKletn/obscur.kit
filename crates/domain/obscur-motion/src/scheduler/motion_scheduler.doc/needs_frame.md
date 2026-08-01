Returns whether another frame is needed — `true` while any motion is still
active (spec §4.17).

After [`advance`](Self::advance) has pruned finished timelines, a non-empty
active set means work remains, so an animation frame is still wanted. This
is the signal the runtime maps to `FrameReason::Animation`.
