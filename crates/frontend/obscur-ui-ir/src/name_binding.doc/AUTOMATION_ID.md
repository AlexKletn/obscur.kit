Mirror of `obscur_view::view::AUTOMATION_ID` — a node's stable identifier for
external automation and assistive tooling (spec §7.5).

Separate from [`TEST_ID`] because the two answer to different owners: a test-id
is the app's own and renames freely, an automation-id is a contract with
software that cannot be updated in the same commit.
