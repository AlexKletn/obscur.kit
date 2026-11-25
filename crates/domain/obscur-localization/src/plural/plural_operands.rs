//! `PluralOperands`: the numeric features a CLDR plural rule is written against
//! (spec §7.2).

#[doc = include_str!("../../docs/api/plural/plural_operands.doc/PluralOperands.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PluralOperands {
    /// `n`: the absolute value of the source number.
    pub n: f64,
    /// `i`: the integer digits of the source number.
    pub i: u64,
    /// `v`: the count of visible fraction digits, with trailing zeros.
    pub v: u32,
    /// `w`: the count of visible fraction digits, without trailing zeros.
    pub w: u32,
    /// `f`: the visible fraction digits with trailing zeros, as an integer.
    pub f: u64,
    /// `t`: the visible fraction digits without trailing zeros, as an integer.
    pub t: u64,
}

impl PluralOperands {
    #[doc = include_str!("../../docs/api/plural/plural_operands.doc/from_integer.md")]
    #[must_use]
    pub fn from_integer(count: i64) -> Self {
        let i = count.unsigned_abs();
        Self {
            // A count beyond 2^53 loses precision in `n`; no rule in CLDR keys on
            // `n` above 99, so this cannot change a category, and the alternative
            // (refusing the count) would be worse.
            #[allow(clippy::cast_precision_loss)]
            n: i as f64,
            i,
            v: 0,
            w: 0,
            f: 0,
            t: 0,
        }
    }

    #[doc = include_str!("../../docs/api/plural/plural_operands.doc/from_decimal.md")]
    #[must_use]
    pub fn from_decimal(text: &str) -> Option<Self> {
        let digits = text.strip_prefix(['-', '+']).unwrap_or(text);
        let (integer, fraction) = match digits.split_once('.') {
            Some((int, frac)) => (int, Some(frac)),
            None => (digits, None),
        };
        if integer.is_empty() || !integer.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        if let Some(frac) = fraction
            && (frac.is_empty() || !frac.bytes().all(|b| b.is_ascii_digit()))
        {
            return None;
        }
        let i: u64 = integer.parse().ok()?;
        let fraction = fraction.unwrap_or("");
        let trimmed = fraction.trim_end_matches('0');
        let f: u64 = if fraction.is_empty() {
            0
        } else {
            fraction.parse().ok()?
        };
        let t: u64 = if trimmed.is_empty() {
            0
        } else {
            trimmed.parse().ok()?
        };
        // `n` is the absolute value: reparse the unsigned spelling rather than
        // reconstructing it from `i` and `f`, which would need the fraction's
        // digit count as a power of ten and lose precision doing it.
        let n: f64 = digits.parse().ok()?;
        Some(Self {
            n,
            i,
            v: u32::try_from(fraction.len()).ok()?,
            w: u32::try_from(trimmed.len()).ok()?,
            f,
            t,
        })
    }
}

impl From<i64> for PluralOperands {
    fn from(count: i64) -> Self {
        Self::from_integer(count)
    }
}

impl From<u64> for PluralOperands {
    fn from(count: u64) -> Self {
        Self {
            #[allow(clippy::cast_precision_loss)]
            n: count as f64,
            i: count,
            v: 0,
            w: 0,
            f: 0,
            t: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TR35's own *Plural Operand Examples* table, transcribed verbatim (minus
    /// the rows with a `c` operand, which this type does not model):
    ///
    /// | source | n | i | v | w | f | t |
    /// |---|---|---|---|---|---|---|
    /// | 1 | 1 | 1 | 0 | 0 | 0 | 0 |
    /// | 1.0 | 1 | 1 | 1 | 0 | 0 | 0 |
    /// | 1.00 | 1 | 1 | 2 | 0 | 0 | 0 |
    /// | 1.3 | 1.3 | 1 | 1 | 1 | 3 | 3 |
    /// | 1.30 | 1.3 | 1 | 2 | 1 | 30 | 3 |
    /// | 1.03 | 1.03 | 1 | 2 | 2 | 3 | 3 |
    /// | 1.230 | 1.23 | 1 | 3 | 2 | 230 | 23 |
    /// | 1200000 | 1200000 | 1200000 | 0 | 0 | 0 | 0 |
    /// | 1200.50 | 1200.5 | 1200 | 2 | 1 | 50 | 5 |
    #[test]
    fn matches_the_tr35_operand_examples_table() {
        let cases: &[(&str, f64, u64, u32, u32, u64, u64)] = &[
            ("1", 1.0, 1, 0, 0, 0, 0),
            ("1.0", 1.0, 1, 1, 0, 0, 0),
            ("1.00", 1.0, 1, 2, 0, 0, 0),
            ("1.3", 1.3, 1, 1, 1, 3, 3),
            ("1.30", 1.3, 1, 2, 1, 30, 3),
            ("1.03", 1.03, 1, 2, 2, 3, 3),
            ("1.230", 1.23, 1, 3, 2, 230, 23),
            ("1200000", 1_200_000.0, 1_200_000, 0, 0, 0, 0),
            ("1200.50", 1200.5, 1200, 2, 1, 50, 5),
        ];
        for &(source, n, i, v, w, f, t) in cases {
            let op = PluralOperands::from_decimal(source).expect(source);
            assert!((op.n - n).abs() < 1e-9, "{source}: n = {} not {n}", op.n);
            assert_eq!(op.i, i, "{source}: i");
            assert_eq!(op.v, v, "{source}: v");
            assert_eq!(op.w, w, "{source}: w");
            assert_eq!(op.f, f, "{source}: f");
            assert_eq!(op.t, t, "{source}: t");
        }
    }

    #[test]
    fn integers_have_no_visible_fraction() {
        let op = PluralOperands::from_integer(21);
        assert_eq!((op.i, op.v, op.w, op.f, op.t), (21, 0, 0, 0, 0));
        assert!((op.n - 21.0).abs() < f64::EPSILON);
    }

    #[test]
    fn n_is_the_absolute_value() {
        assert_eq!(
            PluralOperands::from_integer(-5),
            PluralOperands::from_integer(5)
        );
        let op = PluralOperands::from_decimal("-1.50").expect("valid");
        assert!((op.n - 1.5).abs() < f64::EPSILON);
        assert_eq!(op.i, 1);
        assert_eq!(op.v, 2);
    }

    #[test]
    fn one_and_one_point_zero_are_different_operands() {
        // The distinction the whole type exists for.
        let int = PluralOperands::from_decimal("1").expect("valid");
        let dec = PluralOperands::from_decimal("1.0").expect("valid");
        assert!((int.n - dec.n).abs() < f64::EPSILON);
        assert_ne!(int.v, dec.v);
    }

    #[test]
    fn rejects_spellings_it_does_not_model() {
        for bad in [
            "", ".", "1.", "abc", "1e6", "1_000", "1 000", "1,5", "--1", "1.2c3",
        ] {
            assert!(PluralOperands::from_decimal(bad).is_none(), "{bad}");
        }
    }
}
