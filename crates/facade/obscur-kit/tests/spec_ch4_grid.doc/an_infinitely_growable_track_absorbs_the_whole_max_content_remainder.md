css-grid-2 §12.5: "For intrinsic maximums: Next distribute extra space to the
growth limits of tracks with intrinsic max track sizing function, to
accommodate these items' min-content contributions. Mark any tracks whose
growth limit changed from infinite to finite in this step as infinitely
growable for the next step." (The spec's own worked outcome: "Grow only the
second track's growth limit, and end up with growth limits = [10, 90].")

Item 2's min-content main size is 30px and its max-content is 100px: the text
is ten characters in a face whose advance is exactly 10px at this size, with a
longest word of three characters.
