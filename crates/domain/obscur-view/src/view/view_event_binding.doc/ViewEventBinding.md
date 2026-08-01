Binds a typed event on a node to the semantic command it should invoke.

Events are typed and flow *up*: a node reports an event, and the bound
command is the typed action carried upward for the runtime to dispatch
(spec §2.3, §3.1). To keep the desired tree plain, comparable data, the
binding is fully decoupled from any runtime: it stores **no** closures,
`Box<dyn Fn>`, or `Rc` handles. Both the event and its target command are
referenced by [`InternedSymbol`]; the actual dispatch, capture/bubble
propagation, and closures live in `obscur-runtime`, not here.
