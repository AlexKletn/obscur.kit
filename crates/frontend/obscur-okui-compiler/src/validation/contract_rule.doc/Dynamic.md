`slot={expr}`. A slot is matched against a declaration at *compile* time,
so a name that only exists at runtime cannot be checked against one — and
nothing downstream would route it either.
