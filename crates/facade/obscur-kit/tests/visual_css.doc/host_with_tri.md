Builds a flex-row host containing a single `width:0; height:0` `content-box`
element carrying `tri_class`, so the classic CSS border-triangle trick lays
out through the *real* path (a flex parent sizes the collapsed child; only its
border widths give it extent).
