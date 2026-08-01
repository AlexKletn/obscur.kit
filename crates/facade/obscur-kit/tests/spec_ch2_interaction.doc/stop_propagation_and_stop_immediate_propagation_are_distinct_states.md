§2.3: "- `stop_propagation`;" and "- `stop_immediate_propagation`;"

The two must not be aliased to one "stopped" flag: under that
implementation the two runs produce identical state, and the difference — a
second same-node listener still firing — is exactly the case no
single-listener test exercises.
