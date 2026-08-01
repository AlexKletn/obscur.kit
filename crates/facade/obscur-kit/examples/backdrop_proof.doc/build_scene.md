Builds the display list: a full-viewport hard-edged checkerboard, then a
`backdrop-filter: blur` boundary + a translucent rounded panel fill (the exact
command shape the coordinator emits: BackdropFilter BEFORE the panel's own bg).
