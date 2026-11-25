A diagnostic sink that discards everything it receives.

The default destination for an engine that has not been given a real sink:
diagnostics are still *produced* — so a failure follows exactly the same code
path whether or not anyone is listening — and then dropped.
