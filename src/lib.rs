//! Eval is a powerful expression evaluator.
//!
//! Supported operators: `!` `!=` `""` `''` `()` `[]` `.` `,` `>` `<` `>=` `<=`
//! `==` `+` `-` `*` `/` `%` `&&` `||` `n..m`.
//!
//! Built-in functions: `min()` `max()` `len()` `is_empty()` `array()`.
//!
//! ## Examples
//!
//! You can do mathematical calculations with supported operators:
//!
//! ```
//! use resolver::{eval, to_value};
//!
//! assert_eq!(eval("1 + 2 + 3"), Ok(to_value(6)));
//! assert_eq!(eval("2 * 2 + 3"), Ok(to_value(7)));
//! assert_eq!(eval("2 / 2 + 3"), Ok(to_value(4.0)));
//! assert_eq!(eval("2 / 2 + 3 / 3"), Ok(to_value(2.0)));
//! ```
//!
//! You can eval with context:
//!
//! ```
//! use resolver::{Expr, to_value};
//!
//! assert_eq!(Expr::new("foo == bar")
//!                .value("foo", true)
//!                .value("bar", true)
//!                .exec(),
//!            Ok(to_value(true)));
//! ```
//!
//! You can access data like javascript by using `.` and `[]`. `[]` supports expression.
//!
//! ```
//! use resolver::{Expr, to_value};
//! use std::collections::HashMap;
//!
//! let mut object = HashMap::new();
//! object.insert("foos", vec!["Hello", "world", "!"]);
//!
//! assert_eq!(Expr::new("object.foos[2-1] == 'world'") // Access field `foos` and index `2-1`
//!                .value("object", object)
//!                .exec(),
//!            Ok(to_value(true)));
//! ```
//!
//! You can eval with function:
//!
//! ```
//! use resolver::{Expr, to_value};
//!
//! assert_eq!(Expr::new("say_hello()")
//!                .function("say_hello", |_| Ok(to_value("Hello world!")))
//!                .exec(),
//!            Ok(to_value("Hello world!")));
//! ```
//!
//! You can create an array with `array()`:
//!
//! ```
//! use resolver::{eval, to_value};
//!
//! assert_eq!(eval("array(1, 2, 3, 4, 5)"), Ok(to_value(vec![1, 2, 3, 4, 5])));
//! ```
//!
//! You can create an integer array with `n..m`:
//!
//! ```
//! use resolver::{eval, to_value};
//!
//! assert_eq!(eval("0..5"), Ok(to_value(vec![0, 1, 2, 3, 4])));
//! ```
//!
//! ## Built-in functions
//!
//! ### min()
//! Accept multiple arguments and return the minimum value.
//!
//! ### max()
//! Accept multiple arguments and return the maximum value.
//!
//! ### len()
//! Accept single arguments and return the length of value. Only accept String, Array, Object and Null.
//!
//! ### is_empty()
//! Accept single arguments and return a boolean. Check whether the value is empty or not.
//!
//! ### array()
//! Accept multiple arguments and return an array.
//!
//!
#![recursion_limit="200"]
#![deny(missing_docs)]

#![forbid(unsafe_code)]
#[macro_use]
extern crate quick_error;
extern crate serde;
extern crate serde_json;

mod math;
mod function;
mod operator;
mod node;
mod tree;
mod error;
mod builtin;
mod expr;

pub use expr::ExecOptions;
use function::ConstFunction;
pub use serde_json::Value;
pub use error::Error;
pub use function::Function;
pub use expr::Expr;

use std::{collections::HashMap, rc::Rc, cell::RefCell};
use serde_json::to_value as json_to_value;
use serde::Serialize;

/// Convert variable to `serde_json::Value`
pub fn to_value<S: Serialize>(v: S) -> Value {
    json_to_value(v).unwrap()
}

/// Custom context.
pub type Context = HashMap<String, Value>;
/// Custom contexts. The value of the last context is searched first.
pub type Contexts = Vec<Context>;
/// Custom functions.
pub type Functions = HashMap<String, Function>;
/// Custom static function. Like fn
pub type ConstFunctions = HashMap<String, ConstFunction>;

/// Evaluates the value of an expression.
pub fn eval(expr: &str) -> Result<Value, Error> {
    Expr::new(expr).compile()?.exec()
}

type Compiled = Box<dyn Fn(&[Context], &Functions, Rc<RefCell<ConstFunctions>>) -> Result<Value, Error>>;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    
    use crate::to_value;
    use crate::error::Error;
    use crate::Expr;
    use crate::tree::Tree;
    use crate::Value;
    use crate::eval;

    #[test]
    fn test_add() {
        assert_eq!(eval("2 + 3"), Ok(to_value(5)));
    }

    #[test]
    fn test_brackets_add() {
        assert_eq!(eval("(2 + 3) + (3 + 5)"), Ok(to_value(13)));
    }

    #[test]
    fn test_brackets_float_add() {
        assert_eq!(eval("(2 + 3.2) + (3 + 5)"), Ok(to_value(13.2)));
    }

    #[test]
    fn test_brackets_float_mul() {
        assert_eq!(eval("(2 + 3.2) * 5"), Ok(to_value(26.0)));
    }

    #[test]
    fn test_brackets_sub() {
        assert_eq!(eval("(4 - 3) * 5"), Ok(to_value(5)));
    }

    #[test]
    fn test_useless_brackets() {
        assert_eq!(eval("2 + 3 + (5)"), Ok(to_value(10)));
    }

    #[test]
    fn test_error_brackets_not_with_function() {
        assert_eq!(eval("5 + ()"), Err(Error::BracketNotWithFunction));
    }

    #[test]
    fn test_deep_brackets() {
        assert_eq!(eval("(2 + (3 + 4) + (6 + (6 + 7)) + 5)"), Ok(to_value(33)));
    }

    #[test]
    fn test_brackets_div() {
        assert_eq!(eval("(4 / (2 + 2)) * 5"), Ok(to_value(5.0)));
    }

    #[test]
    fn test_min() {
        assert_eq!(eval("min(30, 5, 245, 20)"), Ok(to_value(5)));
    }

    #[test]
    fn test_min_brackets() {
        assert_eq!(
            eval("(min(30, 5, 245, 20) * 10 + (5 + 5) * 5)"),
            Ok(to_value(100))
        );
    }

    #[test]
    fn test_min_and_mul() {
        assert_eq!(eval("min(30, 5, 245, 20) * 10"), Ok(to_value(50)));
    }

    #[test]
    fn test_max() {
        assert_eq!(eval("max(30, 5, 245, 20)"), Ok(to_value(245)));
    }

    #[test]
    fn test_max_brackets() {
        assert_eq!(
            eval("(max(30, 5, 245, 20) * 10 + (5 + 5) * 5)"),
            Ok(to_value(2500))
        );
    }

    #[test]
    fn test_max_and_mul() {
        assert_eq!(eval("max(30, 5, 245, 20) * 10"), Ok(to_value(2450)));
    }

    #[test]
    fn test_len_array() {
        assert_eq!(eval("len(array(2, 3, 4, 5, 6))"), Ok(to_value(5)));
    }

    #[test]
    fn test_null_and_number() {
        assert_eq!(eval("hos != 0"), Ok(to_value(true)));
        assert_eq!(eval("hos > 0"), Ok(to_value(false)));
    }

    #[test]
    fn test_len_string() {
        assert_eq!(eval("len('Hello world!')"), Ok(to_value(12)));
    }

    #[test]
    fn test_len_object() {
        let mut object = HashMap::new();
        object.insert("field1", "value1");
        object.insert("field2", "value2");
        object.insert("field3", "value3");
        assert_eq!(
            Expr::new("len(object)").value("object", object).exec(),
            Ok(to_value(3_i64))
        );
    }

    #[test]
    fn test_brackets_1() {
        assert_eq!(eval("(5) + (min(3, 4, 5)) + 20"), Ok(to_value(28)));
    }

    #[test]
    fn test_brackets_2() {
        assert_eq!(eval("(((5) / 5))"), Ok(to_value(1.0)));
    }

    #[test]
    fn test_string_add() {
        assert_eq!(eval(r#""Hello"+", world!""#), Ok(to_value("Hello, world!")));
    }

    #[test]
    fn test_equal() {
        assert_eq!(eval("1 == 1"), Ok(to_value(true)));
    }

    #[test]
    fn test_not_equal() {
        assert_eq!(eval("1 != 2"), Ok(to_value(true)));
    }

    #[test]
    fn test_multiple_equal() {
        assert_eq!(eval("(1 == 2) == (2 == 3)"), Ok(to_value(true)));
    }

    #[test]
    fn test_multiple_not_equal() {
        assert_eq!(eval("(1 != 2) == (2 != 3)"), Ok(to_value(true)));
    }

    #[test]
    fn test_greater_than() {
        assert_eq!(eval("1 > 2"), Ok(to_value(false)));
        assert_eq!(eval("2 > 1"), Ok(to_value(true)));
    }

    #[test]
    fn test_less_than() {
        assert_eq!(eval("2 < 1"), Ok(to_value(false)));
        assert_eq!(eval("1 < 2"), Ok(to_value(true)));
    }

    #[test]
    fn test_greater_and_less() {
        assert_eq!(eval("(2 > 1) == (1 < 2)"), Ok(to_value(true)));
    }

    #[test]
    fn test_ge() {
        assert_eq!(eval("2 >= 1"), Ok(to_value(true)));
        assert_eq!(eval("2 >= 2"), Ok(to_value(true)));
        assert_eq!(eval("2 >= 3"), Ok(to_value(false)));
    }

    #[test]
    fn test_le() {
        assert_eq!(eval("2 <= 1"), Ok(to_value(false)));
        assert_eq!(eval("2 <= 2"), Ok(to_value(true)));
        assert_eq!(eval("2 <= 3"), Ok(to_value(true)));
    }

    #[test]
    fn test_quotes() {
        assert_eq!(eval(r#""1><2" + "3<>4""#), Ok(to_value("1><23<>4")));
        assert_eq!(eval(r#""1==2" + "3--4""#), Ok(to_value("1==23--4")));
        assert_eq!(eval(r#""1!=2" + "3>>4""#), Ok(to_value("1!=23>>4")));
        assert_eq!(eval(r#""><1!=2" + "3>>4""#), Ok(to_value("><1!=23>>4")));
    }

    #[test]
    fn test_single_quote() {
        assert_eq!(eval(r#"'1><2' + '3<>4'"#), Ok(to_value("1><23<>4")));
        assert_eq!(eval(r#"'1==2' + '3--4'"#), Ok(to_value("1==23--4")));
        assert_eq!(eval(r#"'1!=2' + '3>>4'"#), Ok(to_value("1!=23>>4")));
        assert_eq!(eval(r#"'!=1<>2' + '3>>4'"#), Ok(to_value("!=1<>23>>4")));
    }

    #[test]
    fn test_single_and_double_quote() {
        assert_eq!(
            eval(r#"' """" ' + ' """" '"#),
            Ok(to_value(r#" """"  """" "#))
        );
    }

    #[test]
    fn test_double_and_single_quote() {
        assert_eq!(
            eval(r#"" '''' " + " '''' ""#),
            Ok(to_value(r#" ''''  '''' "#))
        );
    }

    #[test]
    fn test_array() {
        assert_eq!(eval("array(1, 2, 3, 4)"), Ok(to_value(vec![1, 2, 3, 4])));
    }

    #[test]
    fn test_range() {
        assert_eq!(eval("0..5"), Ok(to_value(vec![0, 1, 2, 3, 4])));
    }

    #[test]
    fn test_range_and_min() {
        assert_eq!(eval("min(0..5)"), Ok(to_value(0)));
    }

    #[test]
    fn test_rem_1() {
        assert_eq!(eval("2 % 2"), Ok(to_value(0)));
    }

    #[test]
    fn test_rem_2() {
        assert_eq!(eval("5 % 56 % 5"), Ok(to_value(0)));
    }

    #[test]
    fn test_rem_3() {
        assert_eq!(eval("5.5 % 23"), Ok(to_value(5.5)));
    }

    #[test]
    fn test_rem_4() {
        assert_eq!(eval("23 % 5.5"), Ok(to_value(1.0)));
    }

    #[test]
    fn test_and_1() {
        assert_eq!(eval("3 > 2 && 2 > 1"), Ok(to_value(true)));
    }

    #[test]
    fn test_and_2() {
        assert_eq!(eval("3 == 2 && 2 == 1"), Ok(to_value(false)));
    }

    #[test]
    fn test_and_3() {
        assert_eq!(eval("3 > 2 && 2 == 1"), Ok(to_value(false)));
    }

    #[test]
    fn test_or_1() {
        assert_eq!(eval("3 > 2 || 2 > 1"), Ok(to_value(true)));
    }

    #[test]
    fn test_or_2() {
        assert_eq!(eval("3 < 2 || 2 < 1"), Ok(to_value(false)));
    }

    #[test]
    fn test_or_3() {
        assert_eq!(eval("3 > 2 || 2 < 1"), Ok(to_value(true)));
    }

    #[test]
    fn test_or_4() {
        assert_eq!(eval("3 < 2 || 2 > 1"), Ok(to_value(true)));
    }

    #[test]
    fn test_not() {
        assert_eq!(eval("!false"), Ok(to_value(true)));
        assert_eq!(eval("!true"), Ok(to_value(false)));
        assert_eq!(eval("!(1 != 2)"), Ok(to_value(false)));
        assert_eq!(eval("!(1 == 2)"), Ok(to_value(true)));
        assert_eq!(eval("!(1 == 2) == true"), Ok(to_value(true)));
    }

    #[test]
    fn test_not_and_brackets() {
        assert_eq!(eval("(!(1 == 2)) == true"), Ok(to_value(true)));
    }

    #[test]
    fn test_object_access() {
        let mut object = HashMap::new();
        object.insert("foo", "Foo, hello world!");
        object.insert("bar", "Bar, hello world!");
        assert_eq!(
            Expr::new("object.foo == 'Foo, hello world!'")
                .value("object", object)
                .exec(),
            Ok(to_value(true))
        );
    }

    #[test]
    fn test_object_dynamic_access() {
        let mut object = HashMap::new();
        object.insert("foo", "Foo, hello world!");
        object.insert("bar", "Bar, hello world!");
        assert_eq!(
            Expr::new("object['foo'] == 'Foo, hello world!'")
                .value("object", object)
                .exec(),
            Ok(to_value(true))
        );
    }

    #[test]
    fn test_object_dynamic_access_2() {
        let mut object = HashMap::new();
        object.insert("foo", "Foo, hello world!");
        object.insert("bar", "Bar, hello world!");
        assert_eq!(
            Expr::new("object[foo] == 'Foo, hello world!'")
                .value("object", object)
                .value("foo", "foo")
                .exec(),
            Ok(to_value(true))
        );
    }

    #[test]
    fn test_path() {
        assert_eq!(Expr::new("array[2-2].foo[2-2]").exec(), Ok(Value::Null));
    }

    #[test]
    fn test_array_access() {
        let array = vec!["hello", "world", "!"];
        assert_eq!(
            Expr::new(
                "array[1-1] == 'hello' && array[1] == 'world' && array[2] == '!'",
            ).value("array", array)
                .exec(),
            Ok(to_value(true))
        );
    }

    #[test]
    fn test_builtin_is_empty() {
        assert_eq!(
            Expr::new("is_empty(array)")
                .value("array", Vec::<String>::new())
                .exec(),
            Ok(to_value(true))
        );
    }

    #[test]
    fn test_builtin_min() {
        assert_eq!(
            Expr::new("min(array)")
                .value("array", vec![23_i32, 34_i32, 45_i32, 2_i32])
                .exec(),
            Ok(to_value(2_i32))
        );
    }

    #[test]
    fn test_custom_function() {
        assert_eq!(
            Expr::new("output()")
                .function(
                    "output",
                    |_| Ok(to_value("This is custom function's output")),
                )
                .exec(),
            Ok(to_value("This is custom function's output"))
        );
    }

    #[test]
    fn test_error_start_with_non_value_operator() {
        let mut tree = Tree {
            raw: "+ + 5".to_owned(),
            ..Default::default()
        };

        tree.parse_pos().unwrap();
        tree.parse_operators().unwrap();

        assert_eq!(tree.parse_node(), Err(Error::StartWithNonValueOperator));
    }

    #[test]
    fn test_error_duplicate_operator() {
        let mut tree = Tree {
            raw: "5 + + 5".to_owned(),
            ..Default::default()
        };

        tree.parse_pos().unwrap();
        tree.parse_operators().unwrap();

        assert_eq!(tree.parse_node(), Err(Error::DuplicateOperatorNode));
    }

    #[test]
    fn test_error_duplicate_value() {
        let mut tree = Tree {
            raw: "2 + 6 5".to_owned(),
            ..Default::default()
        };

        tree.parse_pos().unwrap();
        tree.parse_operators().unwrap();

        assert_eq!(tree.parse_node(), Err(Error::DuplicateValueNode));
    }

    #[test]
    fn test_error_unpaired_brackets() {
        let mut tree = Tree {
            raw: "(2 + 3)) * 5".to_owned(),
            ..Default::default()
        };

        tree.parse_pos().unwrap();

        assert_eq!(tree.parse_operators(), Err(Error::UnpairedBrackets));
    }

    #[test]
    fn test_error_comma() {
        let mut tree = Tree {
            raw: ", 2 + 5".to_owned(),
            ..Default::default()
        };

        tree.parse_pos().unwrap();
        tree.parse_operators().unwrap();

        assert_eq!(tree.parse_node(), Err(Error::CommaNotWithFunction));
    }

    #[test]
    fn test_eval_issue_2() {
        assert_eq!(eval("2 * (4 + 0) + 4"), Ok(to_value(12)));
        assert_eq!(eval("2 * (2 + 2) + (1 + 3)"), Ok(to_value(12)));
        assert_eq!(eval("2 * (4) + (4)"), Ok(to_value(12)));
    }

    #[test]
    fn test_eval_math_function(){
        fn pow(v: Vec<Value>)->Result<Value, Error>{
            let Some(base) = v.get(0) else {
                return Err(Error::ArgumentsLess(2));
            };
            let Some(pow) = v.get(1) else {
                return Err(Error::ArgumentsLess(2));
            };
            let Value::Number(base) = base else {
                return Err(Error::ExpectedNumber);
            };
            let Value::Number(pow) = pow else {
                return Err(Error::ExpectedNumber);
            };
            let Some(base) = base.as_i64() else {
                return Err(Error::Custom("Must can into i64".into()));
            };
            let Some(pow) = pow.as_u64() else {
                return Err(Error::Custom("Must can into u64".into()));
            };
            Ok(base.pow(pow as u32).into())
        }
        fn add2(v: Vec<Value>)->Result<Value, Error>{
            let Some(base) = v.get(0) else {
                return Err(Error::ArgumentsLess(1));
            }; 
            let Value::Number(base) = base else {
                return Err(Error::ExpectedNumber);
            };
            let Some(base) = base.as_i64() else {
                return Err(Error::Custom("Must can into i64".into()));
            };
            Ok((base + 2).into())
        }
        let e = Expr::new("add2(pow(2, 2) + pow(2, 2))").const_function("pow", pow).const_function("add2",add2);
        assert_eq!(e.compile().unwrap().exec(), Ok(to_value(4 + 4 + 2)));
    }

    #[test]
    fn test_div_by_zero() {
        use crate::eval;
        assert_eq!(eval("5 / 0"), Err(Error::DivisionByZero));
    }

    #[test]
    fn test_rem_by_zero() {
        use crate::eval;
        assert_eq!(eval("5 % 0"), Err(Error::ModuloByZero));
    }

    #[test]
    fn test_short_circuit_and() {
        use crate::eval;
        // right side would produce DivisionByZero if evaluated
        assert_eq!(eval("false && (1 / 0 > 0)"), Ok(to_value(false)));
    }

    #[test]
    fn test_short_circuit_or() {
        use crate::eval;
        // right side would produce DivisionByZero if evaluated
        assert_eq!(eval("true || (1 / 0 > 0)"), Ok(to_value(true)));
    }

    #[test]
    fn test_empty_string_double_quotes() {
        use crate::eval;
        assert_eq!(eval(r#""""#), Ok(to_value("")));
    }

    #[test]
    fn test_empty_string_single_quotes() {
        use crate::eval;
        assert_eq!(eval("''"), Ok(to_value("")));
    }

    #[test]
    fn test_not_non_boolean() {
        use crate::eval;
        // !(2 + 3) forces Not to operate on a numeric result
        assert_eq!(eval("!(2 + 3)"), Err(Error::ExpectedBoolean(to_value(5_u64))));
    }

    #[test]
    fn test_len_boolean() {
        assert!(Expr::new("len(v)").value("v", true).exec().is_err());
    }

    #[test]
    fn test_string_gt() {
        use crate::eval;
        assert!(eval("'b' > 'a'").is_err());
    }

    #[test]
    fn test_min_no_args() {
        use crate::eval;
        assert_eq!(eval("min()"), Err(Error::ArgumentsLess(1)));
    }

    #[test]
    fn test_is_empty_null() {
        use crate::eval;
        // unset variable resolves to Null, is_empty(Null) should be true
        assert_eq!(eval("is_empty(hos)"), Ok(to_value(true)));
    }

    #[test]
    fn test_is_empty_number() {
        assert_eq!(
            Expr::new("is_empty(v)").value("v", 0_i32).exec(),
            Ok(to_value(false))
        );
    }

    #[test]
    fn test_chained_dot_access() {
        let mut inner = HashMap::new();
        inner.insert("b", "deep");
        let mut outer = HashMap::new();
        outer.insert("a", to_value(inner));
        assert_eq!(
            Expr::new("obj.a.b").value("obj", outer).exec(),
            Ok(to_value("deep"))
        );
    }

    #[test]
    fn test_and_non_boolean() {
        use crate::eval;
        assert!(eval("1 && true").is_err());
    }

    #[test]
    fn test_or_non_boolean() {
        use crate::eval;
        assert!(eval("1 || false").is_err());
    }

    #[test]
    fn test_null_arithmetic() {
        use crate::eval;
        assert!(eval("hos + 1").is_err());
    }

    #[test]
    fn test_div_float_by_zero() {
        use crate::eval;
        assert_eq!(eval("5.0 / 0.0"), Err(Error::DivisionByZero));
    }

    #[test]
    fn test_rem_float_by_zero() {
        use crate::eval;
        assert_eq!(eval("5.5 % 0.0"), Err(Error::ModuloByZero));
    }

    // ── Unary minus ──────────────────────────────────────────────────────────

    #[test]
    fn test_unary_minus_literal() {
        assert_eq!(eval("-5"), Ok(to_value(-5_i64)));
    }

    #[test]
    fn test_unary_minus_float() {
        assert_eq!(eval("-2.5"), Ok(to_value(-2.5_f64)));
    }

    #[test]
    fn test_unary_minus_double() {
        assert_eq!(eval("-(-3)"), Ok(to_value(3_i64)));
    }

    #[test]
    fn test_unary_minus_variable() {
        assert_eq!(Expr::new("-x").value("x", 5_i32).exec(), Ok(to_value(-5_i64)));
    }

    #[test]
    fn test_unary_minus_in_add() {
        assert_eq!(eval("10 + -3"), Ok(to_value(7_i64)));
    }

    #[test]
    fn test_unary_minus_in_mul() {
        assert_eq!(eval("10 * -2"), Ok(to_value(-20_i64)));
    }

    #[test]
    fn test_unary_minus_pow_precedence() {
        // -2 ** 2 should be -(2**2) = -4.0, not (-2)**2 = 4
        assert_eq!(eval("-2 ** 2"), Ok(to_value(-4.0_f64)));
    }

    // ── Exponentiation (**) ──────────────────────────────────────────────────

    #[test]
    fn test_pow_operator_basic() {
        assert_eq!(eval("2 ** 10"), Ok(to_value(1024.0_f64)));
    }

    #[test]
    fn test_pow_operator_zero_exp() {
        assert_eq!(eval("5 ** 0"), Ok(to_value(1.0_f64)));
    }

    #[test]
    fn test_pow_operator_fractional() {
        assert_eq!(eval("4 ** 0.5"), Ok(to_value(2.0_f64)));
    }

    #[test]
    fn test_pow_operator_priority_add() {
        // 2 + 3 ** 2 == 2 + 9 == 11
        assert_eq!(eval("2 + 3 ** 2"), Ok(to_value(11.0_f64)));
    }

    #[test]
    fn test_pow_operator_priority_mul() {
        // 2 * 3 ** 2 == 2 * 9 == 18
        assert_eq!(eval("2 * 3 ** 2"), Ok(to_value(18.0_f64)));
    }

    #[test]
    fn test_pow_paren_base() {
        assert_eq!(eval("(-2) ** 2"), Ok(to_value(4.0_f64)));
    }

    // ── Null-coalescing (??) ─────────────────────────────────────────────────

    #[test]
    fn test_null_coalesce_null_var() {
        assert_eq!(
            Expr::new("missing ?? 'default'").exec(),
            Ok(to_value("default"))
        );
    }

    #[test]
    fn test_null_coalesce_present_var() {
        assert_eq!(
            Expr::new("v ?? 'default'").value("v", "hello").exec(),
            Ok(to_value("hello"))
        );
    }

    #[test]
    fn test_null_coalesce_zero_not_null() {
        assert_eq!(
            Expr::new("v ?? 99").value("v", 0_i32).exec(),
            Ok(to_value(0_i64))
        );
    }

    #[test]
    fn test_null_coalesce_false_not_null() {
        assert_eq!(
            Expr::new("v ?? true").value("v", false).exec(),
            Ok(to_value(false))
        );
    }

    #[test]
    fn test_null_coalesce_chained() {
        // both sides null -> final fallback
        assert_eq!(eval("missing ?? another ?? 42"), Ok(to_value(42_i64)));
    }

    // ── `in` operator ────────────────────────────────────────────────────────

    #[test]
    fn test_in_array_found() {
        assert_eq!(eval("1 in array(1, 2, 3)"), Ok(to_value(true)));
    }

    #[test]
    fn test_in_array_not_found() {
        assert_eq!(eval("5 in array(1, 2, 3)"), Ok(to_value(false)));
    }

    #[test]
    fn test_in_array_variable() {
        assert_eq!(
            Expr::new("x in arr")
                .value("x", 2_i32)
                .value("arr", vec![1_i32, 2, 3])
                .exec(),
            Ok(to_value(true))
        );
    }

    #[test]
    fn test_in_object_key_found() {
        let mut map = HashMap::new();
        map.insert("foo", 1_i32);
        map.insert("bar", 2_i32);
        assert_eq!(
            Expr::new("'foo' in obj").value("obj", map).exec(),
            Ok(to_value(true))
        );
    }

    #[test]
    fn test_in_object_key_not_found() {
        let mut map = HashMap::new();
        map.insert("foo", 1_i32);
        assert_eq!(
            Expr::new("'baz' in obj").value("obj", map).exec(),
            Ok(to_value(false))
        );
    }

    #[test]
    fn test_in_string_found() {
        assert_eq!(eval("'lo' in 'hello'"), Ok(to_value(true)));
    }

    #[test]
    fn test_in_string_not_found() {
        assert_eq!(eval("'xyz' in 'hello'"), Ok(to_value(false)));
    }

    #[test]
    fn test_in_with_and() {
        assert_eq!(
            eval("1 in array(1,2,3) && 2 in array(2,3,4)"),
            Ok(to_value(true))
        );
    }

    // ── Math builtins ────────────────────────────────────────────────────────

    #[test]
    fn test_abs_negative() {
        assert_eq!(eval("abs(-7)"), Ok(to_value(7_i64)));
    }

    #[test]
    fn test_abs_positive() {
        assert_eq!(eval("abs(3)"), Ok(to_value(3_i64)));
    }

    #[test]
    fn test_abs_float() {
        assert_eq!(eval("abs(-3.5)"), Ok(to_value(3.5_f64)));
    }

    #[test]
    fn test_floor() {
        assert_eq!(eval("floor(3.9)"),  Ok(to_value(3.0_f64)));
        assert_eq!(eval("floor(-3.1)"), Ok(to_value(-4.0_f64)));
    }

    #[test]
    fn test_ceil() {
        assert_eq!(eval("ceil(3.1)"),  Ok(to_value(4.0_f64)));
        assert_eq!(eval("ceil(-3.9)"), Ok(to_value(-3.0_f64)));
    }

    #[test]
    fn test_round() {
        assert_eq!(eval("round(3.5)"), Ok(to_value(4.0_f64)));
        assert_eq!(eval("round(3.4)"), Ok(to_value(3.0_f64)));
    }

    #[test]
    fn test_sqrt() {
        assert_eq!(eval("sqrt(16.0)"), Ok(to_value(4.0_f64)));
        assert_eq!(eval("sqrt(2.0)"),  Ok(to_value(2.0_f64.sqrt())));
    }

    #[test]
    fn test_pow_builtin() {
        assert_eq!(eval("pow(2, 8)"), Ok(to_value(256.0_f64)));
        assert_eq!(eval("pow(3, 0)"), Ok(to_value(1.0_f64)));
    }

    // ── String builtins ──────────────────────────────────────────────────────

    #[test]
    fn test_contains_string_found() {
        assert_eq!(eval("contains('hello world', 'world')"), Ok(to_value(true)));
    }

    #[test]
    fn test_contains_string_not_found() {
        assert_eq!(eval("contains('hello', 'xyz')"), Ok(to_value(false)));
    }

    #[test]
    fn test_contains_array_found() {
        assert_eq!(eval("contains(array(1,2,3), 2)"), Ok(to_value(true)));
    }

    #[test]
    fn test_contains_array_not_found() {
        assert_eq!(eval("contains(array(1,2,3), 9)"), Ok(to_value(false)));
    }

    #[test]
    fn test_starts_with_true() {
        assert_eq!(eval("starts_with('hello', 'hel')"), Ok(to_value(true)));
    }

    #[test]
    fn test_starts_with_false() {
        assert_eq!(eval("starts_with('hello', 'llo')"), Ok(to_value(false)));
    }

    #[test]
    fn test_ends_with_true() {
        assert_eq!(eval("ends_with('hello', 'llo')"), Ok(to_value(true)));
    }

    #[test]
    fn test_ends_with_false() {
        assert_eq!(eval("ends_with('hello', 'hel')"), Ok(to_value(false)));
    }

    #[test]
    fn test_upper() {
        assert_eq!(eval("upper('hello')"), Ok(to_value("HELLO")));
    }

    #[test]
    fn test_lower() {
        assert_eq!(eval("lower('HELLO')"), Ok(to_value("hello")));
    }

    #[test]
    fn test_trim() {
        assert_eq!(eval("trim('  hello  ')"), Ok(to_value("hello")));
        assert_eq!(eval("trim('no-spaces')"),  Ok(to_value("no-spaces")));
    }

    // ── Type-checking builtins ───────────────────────────────────────────────

    #[test]
    fn test_is_null_of_null_value() {
        assert_eq!(
            Expr::new("is_null(v)").value("v", Value::Null).exec(),
            Ok(to_value(true))
        );
    }

    #[test]
    fn test_is_null_unset_var() {
        assert_eq!(eval("is_null(missing)"), Ok(to_value(true)));
    }

    #[test]
    fn test_is_null_false() {
        assert_eq!(eval("is_null(5)"), Ok(to_value(false)));
    }

    #[test]
    fn test_is_number_int() {
        assert_eq!(eval("is_number(42)"), Ok(to_value(true)));
    }

    #[test]
    fn test_is_number_float() {
        assert_eq!(eval("is_number(3.14)"), Ok(to_value(true)));
    }

    #[test]
    fn test_is_number_false() {
        assert_eq!(eval("is_number('hello')"), Ok(to_value(false)));
    }

    #[test]
    fn test_is_string_true() {
        assert_eq!(eval("is_string('hello')"), Ok(to_value(true)));
    }

    #[test]
    fn test_is_string_false() {
        assert_eq!(eval("is_string(42)"), Ok(to_value(false)));
    }

    #[test]
    fn test_is_array_true() {
        assert_eq!(eval("is_array(array(1,2,3))"), Ok(to_value(true)));
    }

    #[test]
    fn test_is_array_false() {
        assert_eq!(eval("is_array(42)"), Ok(to_value(false)));
    }

    // ── Type-conversion builtins ─────────────────────────────────────────────

    #[test]
    fn test_int_from_string() {
        assert_eq!(eval("int('42')"), Ok(to_value(42_i64)));
    }

    #[test]
    fn test_int_from_float_string() {
        assert_eq!(eval("int('3.9')"), Ok(to_value(3_i64)));
    }

    #[test]
    fn test_int_from_number() {
        assert_eq!(eval("int(7)"), Ok(to_value(7_i64)));
    }

    #[test]
    fn test_int_from_bool() {
        assert_eq!(eval("int(true)"),  Ok(to_value(1_i64)));
        assert_eq!(eval("int(false)"), Ok(to_value(0_i64)));
    }

    #[test]
    fn test_float_from_string() {
        assert_eq!(eval("float('3.14')"), Ok(to_value(3.14_f64)));
    }

    #[test]
    fn test_float_from_number() {
        assert_eq!(eval("float(5)"), Ok(to_value(5.0_f64)));
    }

    #[test]
    fn test_str_from_number() {
        assert_eq!(eval("str(42)"), Ok(to_value("42")));
    }

    #[test]
    fn test_str_from_bool() {
        assert_eq!(eval("str(true)"), Ok(to_value("true")));
    }

    #[test]
    fn test_str_from_null() {
        assert_eq!(eval("str(missing)"), Ok(to_value("null")));
    }

    #[test]
    fn test_str_already_string() {
        assert_eq!(eval("str('hello')"), Ok(to_value("hello")));
    }

    // ── Operator precedence edge cases ───────────────────────────────────────

    #[test]
    fn test_and_or_precedence() {
        // && binds tighter than ||: false || (true && false) = false
        assert_eq!(eval("false || true && false"), Ok(to_value(false)));
    }

    #[test]
    fn test_and_or_precedence_2() {
        // true || (false && false) = true
        assert_eq!(eval("true || false && false"), Ok(to_value(true)));
    }

    #[test]
    fn test_in_with_or() {
        assert_eq!(
            eval("5 in array(1,2,3) || 2 in array(2,3,4)"),
            Ok(to_value(true))
        );
    }

    #[test]
    fn test_null_coalesce_with_and() {
        // ?? (priority 1) < && (priority 4), so `a ?? (b && c)`
        assert_eq!(
            Expr::new("missing ?? true && false").exec(),
            Ok(to_value(false))
        );
    }

    #[test]
    fn test_null_coalesce_null_literal() {
        // both sides null
        assert_eq!(eval("missing ?? another"), Ok(Value::Null));
    }

    // ── Exponentiation edge cases ────────────────────────────────────────────

    #[test]
    fn test_pow_zero_zero() {
        // 0 ** 0 = 1 (standard IEEE 754 behaviour)
        assert_eq!(eval("0 ** 0"), Ok(to_value(1.0_f64)));
    }

    #[test]
    fn test_pow_negative_exponent() {
        // 2 ** -1 = 0.5
        assert_eq!(eval("2 ** -1"), Ok(to_value(0.5_f64)));
    }

    #[test]
    fn test_pow_chained_left_assoc() {
        // left-associative: (2 ** 3) ** 2 = 8 ** 2 = 64
        assert_eq!(eval("2 ** 3 ** 2"), Ok(to_value(64.0_f64)));
    }

    // ── Comparison edge cases ────────────────────────────────────────────────

    #[test]
    fn test_null_equals_null() {
        // two unset variables both resolve to null; null == null is true
        assert_eq!(eval("missing == another"), Ok(to_value(true)));
    }

    #[test]
    fn test_cross_type_equality() {
        // number vs string: not equal, no error
        assert_eq!(eval("1 == '1'"), Ok(to_value(false)));
    }

    #[test]
    fn test_null_not_equal_to_zero() {
        assert_eq!(eval("missing != 0"), Ok(to_value(true)));
    }

    // ── `in` error cases ─────────────────────────────────────────────────────

    #[test]
    fn test_in_unsupported_rhs_type() {
        // right-hand side is a number, not array/object/string -> error
        assert!(eval("1 in 42").is_err());
    }

    #[test]
    fn test_in_object_non_string_key() {
        // left-hand side is not a string when right is an object -> error
        let mut map = HashMap::new();
        map.insert("foo", 1_i32);
        assert!(Expr::new("42 in obj").value("obj", map).exec().is_err());
    }

    // ── Array/index edge cases ───────────────────────────────────────────────

    #[test]
    fn test_array_out_of_bounds() {
        // accessing index beyond the end returns null
        let arr = vec![1_i32, 2, 3];
        assert_eq!(
            Expr::new("arr[10]").value("arr", arr).exec(),
            Ok(Value::Null)
        );
    }

    #[test]
    fn test_array_negative_index() {
        // negative index produces a descriptive error, not a panic
        let arr = vec![1_i32, 2, 3];
        assert!(Expr::new("arr[-1]").value("arr", arr).exec().is_err());
    }

    #[test]
    fn test_len_empty_string() {
        assert_eq!(eval("len('')"), Ok(to_value(0_i64)));
    }

    #[test]
    fn test_len_empty_array() {
        assert_eq!(eval("len(array())"), Ok(to_value(0_i64)));
    }

    #[test]
    fn test_is_empty_empty_string() {
        assert_eq!(eval("is_empty('')"), Ok(to_value(true)));
    }

    #[test]
    fn test_is_empty_nonempty_array() {
        assert_eq!(eval("is_empty(array(1))"), Ok(to_value(false)));
    }

    #[test]
    fn test_is_empty_empty_array() {
        assert_eq!(eval("is_empty(array())"), Ok(to_value(true)));
    }

    // ── FunctionNotExists error ──────────────────────────────────────────────

    #[test]
    fn test_function_not_exists() {
        assert_eq!(eval("no_such_fn(1)"), Err(Error::FunctionNotExists("no_such_fn".into())));
    }

    // ── Math builtin edge cases ──────────────────────────────────────────────

    #[test]
    fn test_abs_zero() {
        assert_eq!(eval("abs(0)"), Ok(to_value(0_i64)));
    }

    #[test]
    fn test_abs_non_number() {
        assert!(eval("abs('hello')").is_err());
    }

    #[test]
    fn test_sqrt_zero() {
        assert_eq!(eval("sqrt(0)"), Ok(to_value(0.0_f64)));
    }

    #[test]
    fn test_round_negative_half() {
        // round-half-away-from-zero: -0.5 rounds to -1.0
        assert_eq!(eval("round(-0.5)"), Ok(to_value(-1.0_f64)));
    }

    #[test]
    fn test_floor_integer_input() {
        assert_eq!(eval("floor(5)"), Ok(to_value(5.0_f64)));
    }

    #[test]
    fn test_ceil_integer_input() {
        assert_eq!(eval("ceil(5)"), Ok(to_value(5.0_f64)));
    }

    // ── String builtin edge cases ────────────────────────────────────────────

    #[test]
    fn test_starts_with_empty_prefix() {
        // empty prefix is always a prefix of any string
        assert_eq!(eval("starts_with('hello', '')"), Ok(to_value(true)));
    }

    #[test]
    fn test_ends_with_empty_suffix() {
        assert_eq!(eval("ends_with('hello', '')"), Ok(to_value(true)));
    }

    #[test]
    fn test_contains_empty_needle() {
        assert_eq!(eval("contains('hello', '')"), Ok(to_value(true)));
    }

    #[test]
    fn test_upper_already_upper() {
        assert_eq!(eval("upper('HELLO')"), Ok(to_value("HELLO")));
    }

    #[test]
    fn test_lower_already_lower() {
        assert_eq!(eval("lower('hello')"), Ok(to_value("hello")));
    }

    #[test]
    fn test_trim_only_leading() {
        assert_eq!(eval("trim('  hi')"), Ok(to_value("hi")));
    }

    #[test]
    fn test_trim_only_trailing() {
        assert_eq!(eval("trim('hi  ')"), Ok(to_value("hi")));
    }

    // ── Type-conversion edge cases ───────────────────────────────────────────

    #[test]
    fn test_int_invalid_string() {
        assert!(eval("int('not-a-number')").is_err());
    }

    #[test]
    fn test_float_invalid_string() {
        assert!(eval("float('not-a-number')").is_err());
    }

    #[test]
    fn test_float_from_bool() {
        assert_eq!(eval("float(true)"),  Ok(to_value(1.0_f64)));
        assert_eq!(eval("float(false)"), Ok(to_value(0.0_f64)));
    }

    #[test]
    fn test_int_from_null() {
        // null cannot be meaningfully converted to int -> error
        assert!(eval("int(missing)").is_err());
    }

    #[test]
    fn test_float_from_null() {
        assert!(eval("float(missing)").is_err());
    }

    // ── Range edge cases ─────────────────────────────────────────────────────

    #[test]
    fn test_range_single_element() {
        assert_eq!(eval("3..4"), Ok(to_value(vec![3_i64])));
    }

    #[test]
    fn test_range_empty() {
        // equal bounds -> empty array
        assert_eq!(eval("5..5"), Ok(to_value(Vec::<i64>::new())));
    }

    #[test]
    fn test_range_inverted_empty() {
        // inverted range produces an empty array (mirrors Rust's start..end semantics)
        assert_eq!(eval("5..2"), Ok(to_value(Vec::<i64>::new())));
    }

    // ── if() conditional ─────────────────────────────────────────────────────

    #[test]
    fn test_if_true_branch() {
        assert_eq!(eval("if(true, 1, 2)"), Ok(to_value(1_u64)));
    }

    #[test]
    fn test_if_false_branch() {
        assert_eq!(eval("if(false, 1, 2)"), Ok(to_value(2_u64)));
    }

    #[test]
    fn test_if_with_expression_condition() {
        assert_eq!(eval("if(3 > 2, 'yes', 'no')"), Ok(to_value("yes")));
        assert_eq!(eval("if(1 > 2, 'yes', 'no')"), Ok(to_value("no")));
    }

    #[test]
    fn test_if_short_circuits_true() {
        // false branch should not be evaluated (division by zero)
        assert_eq!(eval("if(true, 42, 1 / 0)"), Ok(to_value(42_u64)));
    }

    #[test]
    fn test_if_short_circuits_false() {
        assert_eq!(eval("if(false, 1 / 0, 99)"), Ok(to_value(99_u64)));
    }

    #[test]
    fn test_if_nested() {
        assert_eq!(eval("if(true, if(false, 1, 2), 3)"), Ok(to_value(2_u64)));
    }

    #[test]
    fn test_if_with_variable() {
        assert_eq!(
            Expr::new("if(x > 0, 'pos', 'non-pos')").value("x", 5_i32).exec(),
            Ok(to_value("pos"))
        );
    }

    #[test]
    fn test_if_non_boolean_condition() {
        assert!(eval("if(1, 'a', 'b')").is_err());
    }

    #[test]
    fn test_if_wrong_arg_count() {
        assert!(eval("if(true, 1)").is_err());
        assert!(eval("if(true, 1, 2, 3)").is_err());
    }

    // ── not in ───────────────────────────────────────────────────────────────

    #[test]
    fn test_not_in_array_true() {
        assert_eq!(eval("5 not in array(1, 2, 3)"), Ok(to_value(true)));
    }

    #[test]
    fn test_not_in_array_false() {
        assert_eq!(eval("2 not in array(1, 2, 3)"), Ok(to_value(false)));
    }

    #[test]
    fn test_not_in_string() {
        assert_eq!(eval("'xyz' not in 'hello'"), Ok(to_value(true)));
        assert_eq!(eval("'lo' not in 'hello'"),  Ok(to_value(false)));
    }

    #[test]
    fn test_not_in_object() {
        let mut map = HashMap::new();
        map.insert("foo", 1_i32);
        assert_eq!(
            Expr::new("'bar' not in obj").value("obj", map).exec(),
            Ok(to_value(true))
        );
    }

    #[test]
    fn test_not_in_with_and() {
        assert_eq!(
            eval("5 not in array(1,2,3) && 9 not in array(2,3,4)"),
            Ok(to_value(true))
        );
    }

    // ── Bitwise operators ────────────────────────────────────────────────────

    #[test]
    fn test_bitwise_and() {
        assert_eq!(eval("12 & 10"), Ok(to_value(8_i64)));   // 1100 & 1010 = 1000
    }

    #[test]
    fn test_bitwise_or() {
        assert_eq!(eval("12 | 10"), Ok(to_value(14_i64)));  // 1100 | 1010 = 1110
    }

    #[test]
    fn test_bitwise_xor() {
        assert_eq!(eval("12 ^ 10"), Ok(to_value(6_i64)));   // 1100 ^ 1010 = 0110
    }

    #[test]
    fn test_bitwise_not() {
        assert_eq!(eval("~0"), Ok(to_value(-1_i64)));
        assert_eq!(eval("~(-1)"), Ok(to_value(0_i64)));
    }

    #[test]
    fn test_shift_left() {
        assert_eq!(eval("1 << 4"), Ok(to_value(16_i64)));
        assert_eq!(eval("3 << 2"), Ok(to_value(12_i64)));
    }

    #[test]
    fn test_shift_right() {
        assert_eq!(eval("16 >> 2"), Ok(to_value(4_i64)));
        assert_eq!(eval("8 >> 1"),  Ok(to_value(4_i64)));
    }

    #[test]
    fn test_bitwise_priority_vs_add() {
        // & (7) binds tighter than + (8)? No: + is 8, & is 7, so + binds tighter.
        // 1 + 2 & 3 = (1 + 2) & 3 = 3 & 3 = 3
        assert_eq!(eval("1 + 2 & 3"), Ok(to_value(3_i64)));
    }

    #[test]
    fn test_bitwise_and_vs_or_priority() {
        // & (7) binds tighter than | (3): 1 | 2 & 3 = 1 | (2 & 3) = 1 | 2 = 3
        assert_eq!(eval("1 | 2 & 3"), Ok(to_value(3_i64)));
    }

    #[test]
    fn test_shift_left_too_large() {
        assert!(eval("1 << 64").is_err());
    }

    #[test]
    fn test_bitwise_on_non_integer() {
        assert!(eval("1.5 & 3").is_err());
    }

    // ── split / join ─────────────────────────────────────────────────────────

    #[test]
    fn test_split_basic() {
        assert_eq!(
            eval("split('a,b,c', ',')"),
            Ok(to_value(vec!["a", "b", "c"]))
        );
    }

    #[test]
    fn test_split_empty_string() {
        assert_eq!(
            eval("split('', ',')"),
            Ok(to_value(vec![""]))
        );
    }

    #[test]
    fn test_split_no_delimiter() {
        assert_eq!(
            eval("split('hello', ',')"),
            Ok(to_value(vec!["hello"]))
        );
    }

    #[test]
    fn test_join_basic() {
        assert_eq!(
            eval("join(array('a', 'b', 'c'), '-')"),
            Ok(to_value("a-b-c"))
        );
    }

    #[test]
    fn test_join_empty_delimiter() {
        assert_eq!(
            eval("join(array('x', 'y'), '')"),
            Ok(to_value("xy"))
        );
    }

    #[test]
    fn test_join_numbers() {
        assert_eq!(
            eval("join(array(1, 2, 3), ', ')"),
            Ok(to_value("1, 2, 3"))
        );
    }

    #[test]
    fn test_split_then_join() {
        assert_eq!(
            eval("join(split('a,b,c', ','), '-')"),
            Ok(to_value("a-b-c"))
        );
    }

    // ── replace ──────────────────────────────────────────────────────────────

    #[test]
    fn test_replace_basic() {
        assert_eq!(eval("replace('hello world', 'world', 'Rust')"), Ok(to_value("hello Rust")));
    }

    #[test]
    fn test_replace_all_occurrences() {
        assert_eq!(eval("replace('aaa', 'a', 'b')"), Ok(to_value("bbb")));
    }

    #[test]
    fn test_replace_not_found() {
        assert_eq!(eval("replace('hello', 'xyz', 'ABC')"), Ok(to_value("hello")));
    }

    #[test]
    fn test_replace_empty_from() {
        // replacing "" inserts the replacement before every char and at end
        assert_eq!(
            eval("replace('ab', '', '-')"),
            Ok(to_value("-a-b-"))
        );
    }

    // ── keys / values ────────────────────────────────────────────────────────

    #[test]
    fn test_keys_basic() {
        let mut map = HashMap::new();
        map.insert("a", 1_i32);
        let result = Expr::new("keys(obj)").value("obj", map).exec().unwrap();
        let mut keys: Vec<String> = result.as_array().unwrap()
            .iter().map(|v| v.as_str().unwrap().to_owned()).collect();
        keys.sort();
        assert_eq!(keys, vec!["a"]);
    }

    #[test]
    fn test_keys_non_object_error() {
        assert!(eval("keys(array(1,2))").is_err());
    }

    #[test]
    fn test_values_basic() {
        let mut map = HashMap::new();
        map.insert("x", 42_i32);
        let result = Expr::new("values(obj)").value("obj", map).exec().unwrap();
        let vals: Vec<i64> = result.as_array().unwrap()
            .iter().map(|v| v.as_i64().unwrap()).collect();
        assert_eq!(vals, vec![42]);
    }

    // ── type_of ──────────────────────────────────────────────────────────────

    #[test]
    fn test_type_of_null() {
        assert_eq!(eval("type_of(missing)"), Ok(to_value("null")));
    }

    #[test]
    fn test_type_of_number() {
        assert_eq!(eval("type_of(42)"),   Ok(to_value("number")));
        assert_eq!(eval("type_of(3.14)"), Ok(to_value("number")));
    }

    #[test]
    fn test_type_of_string() {
        assert_eq!(eval("type_of('hi')"), Ok(to_value("string")));
    }

    #[test]
    fn test_type_of_bool() {
        assert_eq!(eval("type_of(true)"),  Ok(to_value("bool")));
        assert_eq!(eval("type_of(false)"), Ok(to_value("bool")));
    }

    #[test]
    fn test_type_of_array() {
        assert_eq!(eval("type_of(array(1,2))"), Ok(to_value("array")));
    }

    // ── clamp ────────────────────────────────────────────────────────────────

    #[test]
    fn test_clamp_in_range() {
        assert_eq!(eval("clamp(5, 1, 10)"), Ok(to_value(5.0_f64)));
    }

    #[test]
    fn test_clamp_below_min() {
        assert_eq!(eval("clamp(-5, 0, 10)"), Ok(to_value(0.0_f64)));
    }

    #[test]
    fn test_clamp_above_max() {
        assert_eq!(eval("clamp(15, 0, 10)"), Ok(to_value(10.0_f64)));
    }

    #[test]
    fn test_clamp_at_boundary() {
        assert_eq!(eval("clamp(0, 0, 10)"),  Ok(to_value(0.0_f64)));
        assert_eq!(eval("clamp(10, 0, 10)"), Ok(to_value(10.0_f64)));
    }

    // ── log / log2 / log10 ───────────────────────────────────────────────────

    #[test]
    fn test_log_e() {
        let result = eval("log(2.718281828)").unwrap();
        let f = result.as_f64().unwrap();
        assert!((f - 1.0).abs() < 1e-6, "log(e) ≈ 1, got {f}");
    }

    #[test]
    fn test_log2_basic() {
        assert_eq!(eval("log2(8)"), Ok(to_value(3.0_f64)));
    }

    #[test]
    fn test_log10_basic() {
        assert_eq!(eval("log10(1000)"), Ok(to_value(3.0_f64)));
    }

    #[test]
    fn test_log_one() {
        assert_eq!(eval("log(1)"), Ok(to_value(0.0_f64)));
    }

    // ── index_of ─────────────────────────────────────────────────────────────

    #[test]
    fn test_index_of_found() {
        assert_eq!(eval("index_of(array(10, 20, 30), 20)"), Ok(to_value(1_i64)));
    }

    #[test]
    fn test_index_of_not_found() {
        assert_eq!(eval("index_of(array(1, 2, 3), 99)"), Ok(to_value(-1_i64)));
    }

    #[test]
    fn test_index_of_first_occurrence() {
        // returns first occurrence only
        assert_eq!(eval("index_of(array(1, 2, 1), 1)"), Ok(to_value(0_i64)));
    }

    // ── sort / reverse / unique ───────────────────────────────────────────────

    #[test]
    fn test_sort_numbers() {
        assert_eq!(
            eval("sort(array(3, 1, 2))"),
            Ok(to_value(vec![1_i64, 2, 3]))
        );
    }

    #[test]
    fn test_sort_strings() {
        assert_eq!(
            eval("sort(array('banana', 'apple', 'cherry'))"),
            Ok(to_value(vec!["apple", "banana", "cherry"]))
        );
    }

    #[test]
    fn test_sort_already_sorted() {
        assert_eq!(
            eval("sort(array(1, 2, 3))"),
            Ok(to_value(vec![1_i64, 2, 3]))
        );
    }

    #[test]
    fn test_reverse_basic() {
        assert_eq!(
            eval("reverse(array(1, 2, 3))"),
            Ok(to_value(vec![3_i64, 2, 1]))
        );
    }

    #[test]
    fn test_reverse_single() {
        assert_eq!(eval("reverse(array(42))"), Ok(to_value(vec![42_i64])));
    }

    #[test]
    fn test_unique_basic() {
        assert_eq!(
            eval("unique(array(1, 2, 1, 3, 2))"),
            Ok(to_value(vec![1_i64, 2, 3]))
        );
    }

    #[test]
    fn test_unique_no_duplicates() {
        assert_eq!(
            eval("unique(array(1, 2, 3))"),
            Ok(to_value(vec![1_i64, 2, 3]))
        );
    }

    #[test]
    fn test_unique_all_same() {
        assert_eq!(eval("unique(array(5, 5, 5))"), Ok(to_value(vec![5_i64])));
    }

    // ── any / all ────────────────────────────────────────────────────────────

    #[test]
    fn test_any_found() {
        assert_eq!(eval("any(array(1, 2, 3), 2)"), Ok(to_value(true)));
    }

    #[test]
    fn test_any_not_found() {
        assert_eq!(eval("any(array(1, 2, 3), 9)"), Ok(to_value(false)));
    }

    #[test]
    fn test_any_empty_array() {
        assert_eq!(eval("any(array(), 1)"), Ok(to_value(false)));
    }

    #[test]
    fn test_all_match() {
        assert_eq!(eval("all(array(2, 2, 2), 2)"), Ok(to_value(true)));
    }

    #[test]
    fn test_all_partial_match() {
        assert_eq!(eval("all(array(2, 2, 3), 2)"), Ok(to_value(false)));
    }

    #[test]
    fn test_all_empty_array() {
        // vacuously true
        assert_eq!(eval("all(array(), 1)"), Ok(to_value(true)));
    }

    // ── format ───────────────────────────────────────────────────────────────

    #[test]
    fn test_format_basic() {
        assert_eq!(eval("format('Hello, {}!', 'world')"), Ok(to_value("Hello, world!")));
    }

    #[test]
    fn test_format_multiple_args() {
        assert_eq!(
            eval("format('{} + {} = {}', 1, 2, 3)"),
            Ok(to_value("1 + 2 = 3"))
        );
    }

    #[test]
    fn test_format_no_placeholders() {
        assert_eq!(eval("format('no placeholders')"), Ok(to_value("no placeholders")));
    }

    #[test]
    fn test_format_with_variable() {
        assert_eq!(
            Expr::new("format('Hello, {}!', name)").value("name", "Alice").exec(),
            Ok(to_value("Hello, Alice!"))
        );
    }

    #[test]
    fn test_format_number() {
        assert_eq!(eval("format('value: {}', 42)"), Ok(to_value("value: 42")));
    }
}
