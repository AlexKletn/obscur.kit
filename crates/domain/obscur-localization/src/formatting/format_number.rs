use icu_decimal::{DecimalFormatter, input::Decimal};
use icu_locale_core::Locale;

use super::NumberFormatError;
use crate::locale::LanguageTag;

/// Formats an ASCII decimal literal using the locale's digits, separators,
/// grouping, and sign. The literal preserves its decimal precision and
/// trailing zeros without a binary floating-point conversion.
pub fn format_number(locale: &LanguageTag, literal: &str) -> Result<String, NumberFormatError> {
    let decimal = literal
        .parse::<Decimal>()
        .map_err(|_| NumberFormatError::InvalidDecimal)?;
    let locale = locale
        .to_string()
        .parse::<Locale>()
        .map_err(|_| NumberFormatError::LocaleUnavailable)?;
    let formatter = DecimalFormatter::try_new(locale.into(), Default::default())
        .map_err(|_| NumberFormatError::LocaleUnavailable)?;
    Ok(formatter.format_to_string(&decimal))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Localizer;

    #[test]
    fn formats_the_same_exact_decimal_in_multiple_numbering_systems() {
        let mut localizer = Localizer::new(LanguageTag::parse("ru-RU").unwrap());
        let literal = "1234567.050";
        assert_eq!(
            localizer.format_number(literal).unwrap(),
            "1\u{a0}234\u{a0}567,050"
        );
        localizer.set_locale(LanguageTag::parse("en-US").unwrap());
        assert_eq!(localizer.format_number(literal).unwrap(), "1,234,567.050");
        localizer.set_locale(LanguageTag::parse("bn-BD").unwrap());
        assert_eq!(localizer.format_number(literal).unwrap(), "১২,৩৪,৫৬৭.০৫০");
        assert_eq!(
            localizer.format_number("not a number"),
            Err(NumberFormatError::InvalidDecimal)
        );
    }
}
