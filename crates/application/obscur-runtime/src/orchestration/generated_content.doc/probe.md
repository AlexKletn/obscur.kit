Records that one selector is about to be matched. Compiles to nothing outside
test builds, where `SELECTOR_PROBES` counts them and makes "this pass never
asked the engine anything" an assertion rather than a hope.
