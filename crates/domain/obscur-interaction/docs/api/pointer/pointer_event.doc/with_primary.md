Returns this event with its §2.4 primary-pointer status set (fluent).

A multi-touch producer marks every contact but the first as non-primary; a
single-pointer host never calls this, because [`new`](Self::new) already
says `true`.
