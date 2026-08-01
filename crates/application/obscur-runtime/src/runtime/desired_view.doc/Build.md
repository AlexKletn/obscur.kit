A tree built on demand from current state, called once, at reconcile time
(after input dispatch). The closure was moved out of the runtime by the
reactive caller, so invoking it does not alias `&mut self`.
