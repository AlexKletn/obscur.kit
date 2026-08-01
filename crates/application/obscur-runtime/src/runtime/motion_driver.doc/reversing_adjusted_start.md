The reversing-adjusted start value: the "logical start state" a reversal
is measured against. For a fresh or non-reversing transition it is the
start value; for a reversal it is the *end* value of the transition it
interrupted (spec §3.1), so repeated reversals compose correctly.
