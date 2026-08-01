The monotonically increasing version of a resource's content (spec §5.15).

Every time a resource is (re)loaded, its version advances. Versioning enables
hot reload and **stale-job rejection**: a load job carries the version it was
issued for, and when it completes the [`crate::hub`] compares that version
against the resource's current version. A job whose version is older than the
resource's current version is rejected as stale, so a superseded reload can
never overwrite a newer result.

This composes [`VersionStamp`] rather than reimplementing it. Version `0` is
reserved to mean *unversioned*; the first real version is
[`ResourceVersion::initial`], which starts at `1`.
