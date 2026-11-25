Formats a decimal amount with locale-specific digits, separators and currency
symbol placement. Pass a three-letter uppercase currency code, such as `USD`
or `JPY`. The formatter can display the code itself when the locale lacks a
symbol. It preserves the input fractional precision, so pass `"12.00"` for two
places or `"12"` for a whole amount.

The formatter applies the number sign inside the currency pattern. For
example, a negative US dollar amount in `en-US` renders as `$-12.00`.

# Errors

Returns an error for malformed amounts, malformed codes, or unavailable locale
formatting data.
