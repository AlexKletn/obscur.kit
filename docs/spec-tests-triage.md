# Spec-derived tests: triage register

`cargo test --workspace --no-fail-fast` is red on purpose. This file records **which
red is which**, because they are not the same thing and must not be acted on the same
way.

State: **5,429 passing, 111 failing.** 310 cases were derived across §1–§7 and §9;
§8 was skipped (67 sections of declared stubs, nothing to test).

## Read this before closing anything

Triage classified the failures into two piles of nearly equal size — **48 real gaps
and 45 bad tests.** That ratio is the headline finding of the rollout, and it is very
different from the §4 pilot, which was 18 real gaps against 4 bad tests.

**The method's precision tracks the spec's precision.** §4 defers to the vendored W3C
texts: normative, numeric, quotable, so a deriver who never saw the code still lands
on the right expected value. §1, §2, §3 and §7 are this project's own prose, and much
of it is a bullet list — "assignment;" under Forbidden, "components produce a desired
declarative tree". A deriver reading that has to invent the observable specifics, and
inventing is exactly what produces a confident wrong test.

So: **the triage step is not optional, and the derivation is not trustworthy without
it.** Four §4 failures read as algorithm bugs in provably correct code; at this scale
there are 45.

## The 45 are not all the same defect

Sorting them by what actually went wrong matters more than the count:

1. **Invented requirements.** The spec says less than the test assumes. Example: §2.10
   lists "assignment" as forbidden; a case additionally demanded that the compiler
   suppress its output artifact, which no sentence requires. Correcting these means
   weakening the assertion to what the spec actually says.
2. **Broken fixtures.** The test's own harness contradicts its premise. Example:
   `tree_key_reorder_preserves_child_state` builds each row with `100 + i` as its
   ComponentId, so identity is a function of *position* — the very thing the test
   claims to hold constant. The implementation is correct; the fixture defeats itself.
3. **Wrong setup.** `the_selection_anchor_persists_across_successive_shift_clicks` uses
   the default selection policy, which is *Single*; collapsing a range is the
   documented, correct behaviour for it. The case wanted Multi and never asked for it.
4. **Sampled too early.** An error-boundary case read one frame before the retry
   §1.7 allows had landed.

Only (1) is a disagreement about what the spec means. (2)–(4) are ordinary test bugs —
and they are a consequence of the isolation, not a failure of it: the deriver never saw
the API, and the translator was forbidden from changing an expected value, so a wrong
fixture got translated faithfully.

## The red list (60)

**Anything red that is not on this list is a regression.**

State: **5,570 passing, 60 failing.** Was 88 before implementation began.

### How to count them, because I got this wrong

Three grep pipelines over the same tree gave 64, 107 and 130. Parallel test binaries
write to one stream and their lines interleave, so counting through a pipe is
unreliable. Capture one run to a file first, then parse:

    cargo test --workspace --no-fail-fast -j6 > run.txt 2>&1 || true

and count unique `^test NAME ... FAILED$` lines. That agrees with the summed
`test result` totals; the pipeline method does not.

### Never build in the scratchpad

`/tmp` here is **tmpfs — it lives in RAM**. An agent created a git worktree under the
scratchpad and built the workspace inside it: 13 GB of `target/` in memory, on a 30 GB
machine that had already been OOM-killed once. Writes then failed with `EDQUOT` and
every command producing output died. Use Рабочий каталог и состояние индекса сохранены WIP on master: db1352f feat(css): the flex shorthand, physical units, selector functions, animation longhands for a baseline, or a worktree
somewhere on disk — never under .

- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
- 
