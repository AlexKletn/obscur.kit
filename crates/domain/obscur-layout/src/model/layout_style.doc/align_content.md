Grid container: *block-axis* (row) distribution of leftover track space
(`align-content`): when the rows' total size is smaller than the content
height the free space is distributed per this value. Ignored by flex.
Defaults to [`JustifyContent::FlexStart`] (no distribution), so a default
grid packs its rows at the top exactly as before.
