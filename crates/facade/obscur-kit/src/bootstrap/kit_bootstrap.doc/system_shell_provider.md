Builds the platform shell provider. On Linux this shells out to
`xdg-open`; the dialog and notification providers are obtained the same
way from `crate::system_platform::provider`.
