Uppercases the first cased letter of each whitespace-bounded word (§2.1
`capitalize`).

A word boundary is any Unicode whitespace run; within a word the first
character for which [`char::is_alphabetic`] holds is title/upper-cased and the
rest are copied verbatim. Leading punctuation or digits do not consume the
"first letter", matching the spec intent that `capitalize` targets the first
*letter* of the word.
