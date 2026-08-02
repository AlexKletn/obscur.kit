The local name a widget's own `view()` sets on its root node, read back the
way the matcher reads it (`StyleAttributes::extract` → `local_name()`) rather
than by looking for a property by symbol.
