The source that produced an event entering the routing model (spec §4.7).

Every event — whether from a real device, an assistive technology, or a test
harness — flows through the same propagation path. Recording its origin lets
handlers distinguish direct device input from accessibility actions
([`Accessibility`](Self::Accessibility)) and from test-injected inputs
([`Synthetic`](Self::Synthetic), spec §7.4), which are otherwise
indistinguishable at the routing layer.
