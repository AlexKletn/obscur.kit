use icu_datetime::{DateTimeFormatter, fieldsets::YMD, input::Date};
use icu_locale_core::Locale;
use writeable::Writeable;

use super::DateFormatError;
use crate::locale::LanguageTag;

/// Formats an ISO calendar date as a medium-length date in the locale's
/// preferred calendar. Month and day are one-based. The input carries
/// no time or time zone.
pub fn format_date(
    locale: &LanguageTag,
    year: i32,
    month: u8,
    day: u8,
) -> Result<String, DateFormatError> {
    let date = Date::try_new_iso(year, month, day).map_err(|_| DateFormatError::InvalidDate)?;
    let locale = locale
        .to_string()
        .parse::<Locale>()
        .map_err(|_| DateFormatError::LocaleUnavailable)?;
    let formatter = DateTimeFormatter::try_new(locale.into(), YMD::medium())
        .map_err(|_| DateFormatError::LocaleUnavailable)?;
    Ok(formatter.format(&date).write_to_string().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Localizer;

    #[test]
    fn formats_the_same_date_with_the_current_locale_and_calendar() {
        let mut localizer = Localizer::new(LanguageTag::parse("en-US").unwrap());
        assert_eq!(localizer.format_date(2025, 1, 15).unwrap(), "Jan 15, 2025");
        localizer.set_locale(LanguageTag::parse("ru-RU").unwrap());
        let russian = localizer.format_date(2025, 1, 15).unwrap();
        assert!(russian.starts_with("15 янв."));
        assert!(russian.contains("2025"));
        localizer.set_locale(LanguageTag::parse("th-TH").unwrap());
        assert!(localizer.format_date(2025, 1, 15).unwrap().contains("2568"));
        assert_eq!(
            localizer.format_date(2025, 2, 29),
            Err(DateFormatError::InvalidDate)
        );
        assert!(localizer.format_date(2024, 2, 29).is_ok());
    }
}
