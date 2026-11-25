A stateless byte codec for the DevTools wire envelopes (spec §7.6).

Every method is independent; the codec holds no state. It encodes and decodes
exactly the two directions the transport carries — [`ServerBound`] toward the
server and [`ClientBound`] toward the client — in the layout documented at
the [module level](self).
