The contract a component id was registered under, if any.

The registry is the only place a mounted node's *declared* shape can be read
from: a `MountedComponent` holds the instance and the erased component, not the
contract its factory reported at registration.
