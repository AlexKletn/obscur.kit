Runs the whole match + style pass: every element is re-matched and every
node re-resolved.

Returns the fresh styles, how many nodes were resolved (all of them), and a
[`StyleDelta::Full`] — the runtime's retained map is untouched, so it is still the
previous frame's map for the layout classifier to diff against.
