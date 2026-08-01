Event bindings (`event → command`) carried on element nodes, copied from
the desired node's events during reconciliation. Actions flow *up*: the
runtime looks a fired event up here to find the command it dispatches
(spec §2.3, §3.1). Empty for non-element nodes and elements with no
events.
