The editing field's visible content width `W` (logical px): its border-box
`border_box_width` minus the left/right padding and border edges — the
scrollport the single-line run scrolls within (CSS Overflow 3 §3.1). Only
absolute `px` padding/border widths are honored (mirroring the scene
coordinator's `border_width_of`); the result is clamped non-negative so a box
smaller than its insets yields `0` rather than a negative width.
