The layout-chrome stylesheet: masthead, section cards, captions, the legend
swatches, and the graph-paper page. Returned as `&'static str` so both drivers
compile it through the real `obscur-css` frontend.

Palette matches the source kit: ink `#2b2a27`, muted `#5a584f`, faint `#9b998f`,
disabled-ink `#b7b5ad`, page `#eceae5`, surface `#faf9f6`, hover `#e4e2db`,
disabled-fill `#f2f0ea`. Fonts are the host's (no IBM Plex), so only sizes and
weights are set — mono captions/subtitles read in the host font by design.
