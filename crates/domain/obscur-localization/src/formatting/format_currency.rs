use icu_decimal::input::Decimal;
use icu_experimental::dimension::currency::{CurrencyCode, formatter::CurrencyFormatter};
use icu_locale_core::Locale;
use writeable::Writeable;

use crate::locale::LanguageTag;

/// An amount, currency code, or locale could not be formatted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrencyFormatError {
    /// The input is not a supported decimal literal.
    InvalidDecimal,
    /// The currency code is not three uppercase ASCII letters.
    InvalidCurrencyCode,
    /// The locale or its currency formatting data could not be used.
    LocaleUnavailable,
}

impl std::fmt::Display for CurrencyFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidDecimal => "invalid decimal amount",
            Self::InvalidCurrencyCode => "invalid currency code",
            Self::LocaleUnavailable => "currency formatting locale unavailable",
        })
    }
}

impl std::error::Error for CurrencyFormatError {}

#[doc = include_str!("../../docs/api/formatting/format_currency.doc/format_currency.md")]
pub fn format_currency(
    locale: &LanguageTag,
    literal: &str,
    currency_code: &str,
) -> Result<String, CurrencyFormatError> {
    let amount = literal
        .parse::<Decimal>()
        .map_err(|_| CurrencyFormatError::InvalidDecimal)?;
    if currency_code.len() != 3 || !currency_code.bytes().all(|byte| byte.is_ascii_uppercase()) {
        return Err(CurrencyFormatError::InvalidCurrencyCode);
    }
    let code = CurrencyCode(
        currency_code
            .parse()
            .map_err(|_| CurrencyFormatError::InvalidCurrencyCode)?,
    );
    let locale = locale
        .to_string()
        .parse::<Locale>()
        .map_err(|_| CurrencyFormatError::LocaleUnavailable)?;
    let formatter = CurrencyFormatter::try_new(locale.into(), Default::default())
        .map_err(|_| CurrencyFormatError::LocaleUnavailable)?;
    let mut output = String::new();
    formatter
        .format_fixed_decimal(&amount, code)
        .write_to(&mut output)
        .map_err(|_| CurrencyFormatError::LocaleUnavailable)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Localizer;

    #[test]
    fn localizes_symbol_placement_grouping_and_digits() {
        let mut localizer = Localizer::new(LanguageTag::parse("en-US").unwrap());
        assert_eq!(
            localizer.format_currency("12345.670", "USD").unwrap(),
            "$12,345.670"
        );
        localizer.set_locale(LanguageTag::parse("fr-FR").unwrap());
        assert_eq!(
            localizer.format_currency("-12345.67", "EUR").unwrap(),
            "-12\u{202f}345,67\u{a0}€"
        );
        localizer.set_locale(LanguageTag::parse("ar-EG").unwrap());
        let formatted = localizer.format_currency("12345.67", "EGP").unwrap();
        assert!(formatted.contains("١٢٬٣٤٥٫٦٧"));
        assert!(formatted.contains("ج.م."));
    }

    #[test]
    fn rejects_invalid_amounts_and_currency_codes() {
        let localizer = Localizer::new(LanguageTag::parse("en-US").unwrap());
        assert_eq!(
            localizer.format_currency("NaN", "USD"),
            Err(CurrencyFormatError::InvalidDecimal)
        );
        assert_eq!(
            localizer.format_currency("12", "usd"),
            Err(CurrencyFormatError::InvalidCurrencyCode)
        );
    }
}
