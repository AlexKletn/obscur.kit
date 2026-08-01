Locates host-installed faces through fontconfig (spec §5.18).

The tool names are held rather than hard-coded so a caller may point at a
different fontconfig build; the defaults are the conventional `fc-match` /
`fc-list`.
