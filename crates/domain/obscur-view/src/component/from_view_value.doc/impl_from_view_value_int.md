Implements [`FromViewValue`] for an integer type via `i64::try_into`, so a
value outside the target's range is a decode failure rather than a wrap.
