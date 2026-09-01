Installs a custom-property set, sharing it rather than copying it.

Writing a property afterwards forks the set (`set_custom_property` is
copy-on-write), so sharing is never observable.
