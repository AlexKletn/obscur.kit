#![doc = include_str!("mod.doc/_module.md")]

mod value_type;
pub use value_type::ValueType;

mod expression_type;
pub use expression_type::ExpressionType;

mod type_error;
pub use type_error::{TypeError, TypeErrorKind};

mod type_checker;
pub use type_checker::TypeChecker;
