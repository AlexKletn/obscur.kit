A reference, no-op [`DialogProvider`] that presents nothing (spec §6.9).

`NullDialogProvider` honestly reports that it can present no dialog:
[`Self::origin`] is `"null"`, [`Self::present`] returns
[`DialogResult::Unsupported`], and [`Self::supports`] returns
[`SupportLevel::Unavailable`] for every request. Its "unsupported" answers are
correct behavior — a documented reference for the honest-capability contract —
not a stub waiting to be filled in.
