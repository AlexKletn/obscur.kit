Reads both axis longhands out of a node's computed style, treating an absent
declaration (or an absent style) as the initial `visible`, and applies §3's
pairing rule. [`None`] answers "visible on both axes", which is what a node with
no style at all means.
