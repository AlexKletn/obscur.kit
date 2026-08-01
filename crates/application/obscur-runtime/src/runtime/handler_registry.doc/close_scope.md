Closes the scope with this `id` **and drops its handlers**, returning
whether one was open. The two go together: a closed dialog whose handlers
survived would keep claiming commands from outside itself.
