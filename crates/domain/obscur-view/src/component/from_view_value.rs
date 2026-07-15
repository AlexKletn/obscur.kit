//! `FromViewValue`: reading one authored [`ViewValue`] as a concrete Rust type.

use crate::view::ViewValue;

/// Reads a single authored [`ViewValue`] as a concrete Rust type (spec §3.1).
///
/// [`ViewValue`] is the small closed enum a `.okui` document or an `okui!` macro
/// produces (`Null`/`Bool`/`Int`/`Float`/`Text`). This trait is the per-type half
/// of prop decoding: [`FromProps`](crate::component::FromProps) finds the value
/// for a property, and this turns it into the field's type.
///
/// # Strict, with one deliberate widening
///
/// Decoding does not coerce across kinds: a `Text` never becomes a `bool`, and a
/// `Bool` never becomes an integer. An authored mistake should surface as a
/// [`PropsError::Type`](crate::component::PropsError::Type), not as a silently
/// wrong value.
///
/// The single exception is **`Int` → floating point**: an author writing
/// `opacity={1}` means `1.0`, and requiring `{1.0}` would be a papercut with no
/// safety benefit. `Float` → integer is *not* accepted (it would have to
/// truncate), and an `Int` outside the target's range is a
/// [`Type`](crate::component::PropsError::Type) error rather than a wrap.
///
/// # Implementing it
///
/// Return [`None`] when the value cannot be read; the caller turns that into the
/// error, so implementors do not need a property name. [`TYPE_NAME`](Self::TYPE_NAME)
/// is what the resulting diagnostic reports as "wanted".
pub trait FromViewValue: Sized {
    /// The name this type reports in a
    /// [`PropsError::Type`](crate::component::PropsError::Type) diagnostic.
    const TYPE_NAME: &'static str;

    /// Reads `value` as `Self`, or [`None`] when it cannot be read.
    fn from_view_value(value: &ViewValue) -> Option<Self>;
}

impl FromViewValue for bool {
    const TYPE_NAME: &'static str = "bool";

    fn from_view_value(value: &ViewValue) -> Option<Self> {
        match value {
            ViewValue::Bool(flag) => Some(*flag),
            _ => None,
        }
    }
}

/// Implements [`FromViewValue`] for an integer type via `i64::try_into`, so a
/// value outside the target's range is a decode failure rather than a wrap.
macro_rules! impl_from_view_value_int {
    ($($ty:ty),* $(,)?) => {
        $(
            impl FromViewValue for $ty {
                const TYPE_NAME: &'static str = stringify!($ty);

                fn from_view_value(value: &ViewValue) -> Option<Self> {
                    match value {
                        ViewValue::Int(int) => Self::try_from(*int).ok(),
                        _ => None,
                    }
                }
            }
        )*
    };
}

impl_from_view_value_int!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl FromViewValue for f64 {
    const TYPE_NAME: &'static str = "f64";

    #[expect(
        clippy::cast_precision_loss,
        reason = "the deliberate Int widening: an authored `{1}` means `1.0`"
    )]
    fn from_view_value(value: &ViewValue) -> Option<Self> {
        match value {
            // A `ViewValue::Float` is already an `f64`: no cast at all.
            ViewValue::Float(float) => Some(*float),
            ViewValue::Int(int) => Some(*int as Self),
            _ => None,
        }
    }
}

impl FromViewValue for f32 {
    const TYPE_NAME: &'static str = "f32";

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        reason = "f32 is a narrower authored type by the author's own choice"
    )]
    fn from_view_value(value: &ViewValue) -> Option<Self> {
        match value {
            ViewValue::Float(float) => Some(*float as Self),
            ViewValue::Int(int) => Some(*int as Self),
            _ => None,
        }
    }
}

impl FromViewValue for String {
    const TYPE_NAME: &'static str = "String";

    fn from_view_value(value: &ViewValue) -> Option<Self> {
        match value {
            ViewValue::Text(text) => Some(text.to_string()),
            _ => None,
        }
    }
}

impl FromViewValue for Box<str> {
    const TYPE_NAME: &'static str = "Box<str>";

    fn from_view_value(value: &ViewValue) -> Option<Self> {
        match value {
            ViewValue::Text(text) => Some(text.clone()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FromViewValue;
    use crate::view::ViewValue;

    #[test]
    fn scalars_read_their_own_kind() {
        assert_eq!(bool::from_view_value(&ViewValue::Bool(true)), Some(true));
        assert_eq!(i64::from_view_value(&ViewValue::Int(-5)), Some(-5));
        assert_eq!(f64::from_view_value(&ViewValue::Float(1.5)), Some(1.5));
        assert_eq!(
            String::from_view_value(&ViewValue::Text("hi".into())),
            Some("hi".to_owned())
        );
        assert_eq!(
            Box::<str>::from_view_value(&ViewValue::Text("hi".into())).as_deref(),
            Some("hi")
        );
    }

    #[test]
    fn kinds_do_not_coerce_into_each_other() {
        // A `.okui` author writing the wrong kind must get an error, not a
        // silently plausible value.
        assert_eq!(bool::from_view_value(&ViewValue::Text("true".into())), None);
        assert_eq!(bool::from_view_value(&ViewValue::Int(1)), None);
        assert_eq!(i64::from_view_value(&ViewValue::Text("5".into())), None);
        assert_eq!(i64::from_view_value(&ViewValue::Bool(true)), None);
        assert_eq!(String::from_view_value(&ViewValue::Int(5)), None);
        assert_eq!(bool::from_view_value(&ViewValue::Null), None);
    }

    #[test]
    fn an_int_widens_into_a_float_but_a_float_never_truncates() {
        // The one deliberate widening: `opacity={1}` means 1.0.
        assert_eq!(f64::from_view_value(&ViewValue::Int(1)), Some(1.0));
        assert_eq!(f32::from_view_value(&ViewValue::Int(2)), Some(2.0));
        // The reverse would have to lose information, so it is refused.
        assert_eq!(i64::from_view_value(&ViewValue::Float(1.0)), None);
        assert_eq!(u32::from_view_value(&ViewValue::Float(1.9)), None);
    }

    #[test]
    fn an_out_of_range_int_fails_instead_of_wrapping() {
        assert_eq!(u8::from_view_value(&ViewValue::Int(255)), Some(255));
        assert_eq!(u8::from_view_value(&ViewValue::Int(256)), None);
        assert_eq!(u32::from_view_value(&ViewValue::Int(-1)), None);
        assert_eq!(i32::from_view_value(&ViewValue::Int(i64::MAX)), None);
    }
}
