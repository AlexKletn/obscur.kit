Maps a fontconfig `%{width}` to the nearest [`FontStretch`].

fontconfig's width scale is already a percentage of normal
(`FC_WIDTH_NORMAL` = 100), matching CSS's, so this only has to snap to the
nearest named step — fontconfig reports whole percents (`63`) where CSS names
fractions (`62.5`).
