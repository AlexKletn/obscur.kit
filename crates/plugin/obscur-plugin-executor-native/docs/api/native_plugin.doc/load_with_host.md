Loads a trusted native package with its optional host-call binding. The host
table is passed to the guest immediately before its initialization callback.

# Safety

All requirements of [Self::load] apply. The bind symbol must use the documented
version-1 signature. The host callback must remain callable for the life of
the process, and must reject expired or unauthorized instance handles.
