Custom paths: `view` is the module holding `View`/`ViewNode`/`ViewValue`,
`identity` the one holding `ComponentId`/`InternedSymbol`.

Both are written into generated code verbatim, so pass absolute paths
(leading `::`) unless the generated file is `include!`d somewhere a
relative one resolves.
