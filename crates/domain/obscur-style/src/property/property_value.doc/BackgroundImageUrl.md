A `url(...)` background image: the value of `background-image` when it is
an authored image reference. The payload is the **raw** URL/path string
exactly as authored (unquoted or with its quotes stripped), *not* resolved
against any base — the runtime hands it to a `ResourceLoader` /
`ResourceDecoder` and, once decoded, the scene coordinator paints the
resulting image stretched over the node's box (spec §5.2, §5.16). A URL
whose image is not yet loaded simply paints nothing (the background-color
shows through).
