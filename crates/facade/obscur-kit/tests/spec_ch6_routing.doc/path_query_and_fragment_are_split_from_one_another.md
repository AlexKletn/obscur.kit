§6.13 Router: "Basic URL-like router:" / "- path;" / "- query;" /
"- fragment;"

Path, query and fragment are split from one another before any of them is
interpreted: no `?` residue in the path, no `#` residue in the last query
value, and the fragment carries no leading `#`.
