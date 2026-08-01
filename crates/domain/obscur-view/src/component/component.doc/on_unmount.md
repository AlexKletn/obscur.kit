Called as the instance is torn down. Default: no-op.

Releasing scoped subscriptions and local state is automatic (it happens
when the instance's scope and fields drop); override this only for
side-effects that must run *before* that cleanup.
