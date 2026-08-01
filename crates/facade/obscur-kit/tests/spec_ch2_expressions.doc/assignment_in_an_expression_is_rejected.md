§2.10, Forbidden: "- assignment;"

WEAKENED 2026-07-19: this originally required the diagnostic to contain the
word "assignment" (`reports(source, "assignment")`). §2.10 enumerates
forbidden expression *forms* and says nothing about diagnostic wording; the
assignment IS refused, with the front end's generic §2.10 message. Candidate
spec amendment: state whether a §2.10 refusal must name the forbidden
category it violated.
