
use std::cmp::Ordering;
use crate::{Function, Functions, Value, to_value};
use crate::math::Math;
use crate::error::Error;

pub struct BuiltIn;

impl BuiltIn {
    pub fn create_builtins() -> Functions {
        let mut functions = Functions::new();
        functions.insert("min".to_owned(), create_min_function());
        functions.insert("max".to_owned(), create_max_function());
        functions.insert("len".to_owned(), create_len_function());
        functions.insert("is_empty".to_owned(), create_is_empty_function());
        functions.insert("array".to_owned(), create_array_function());
        // Math
        functions.insert("abs".to_owned(), create_abs_function());
        functions.insert("floor".to_owned(), create_floor_function());
        functions.insert("ceil".to_owned(), create_ceil_function());
        functions.insert("round".to_owned(), create_round_function());
        functions.insert("sqrt".to_owned(), create_sqrt_function());
        functions.insert("pow".to_owned(), create_pow_function());
        functions.insert("clamp".to_owned(), create_clamp_function());
        functions.insert("log".to_owned(), create_log_function());
        functions.insert("log2".to_owned(), create_log2_function());
        functions.insert("log10".to_owned(), create_log10_function());
        // String
        functions.insert("contains".to_owned(), create_contains_function());
        functions.insert("starts_with".to_owned(), create_starts_with_function());
        functions.insert("ends_with".to_owned(), create_ends_with_function());
        functions.insert("upper".to_owned(), create_upper_function());
        functions.insert("lower".to_owned(), create_lower_function());
        functions.insert("trim".to_owned(), create_trim_function());
        functions.insert("replace".to_owned(), create_replace_function());
        functions.insert("split".to_owned(), create_split_function());
        functions.insert("join".to_owned(), create_join_function());
        functions.insert("format".to_owned(), create_format_function());
        // Type checks
        functions.insert("is_null".to_owned(), create_is_null_function());
        functions.insert("is_number".to_owned(), create_is_number_function());
        functions.insert("is_string".to_owned(), create_is_string_function());
        functions.insert("is_array".to_owned(), create_is_array_function());
        functions.insert("type_of".to_owned(), create_type_of_function());
        // Type conversion
        functions.insert("int".to_owned(), create_int_function());
        functions.insert("float".to_owned(), create_float_function());
        functions.insert("str".to_owned(), create_str_function());
        // Array / object inspection
        functions.insert("keys".to_owned(), create_keys_function());
        functions.insert("values".to_owned(), create_values_function());
        functions.insert("index_of".to_owned(), create_index_of_function());
        functions.insert("sort".to_owned(), create_sort_function());
        functions.insert("reverse".to_owned(), create_reverse_function());
        functions.insert("unique".to_owned(), create_unique_function());
        functions.insert("any".to_owned(), create_any_function());
        functions.insert("all".to_owned(), create_all_function());
        functions
    }
}

#[derive(PartialEq)]
enum Compare {
    Min,
    Max,
}

fn create_min_function() -> Function {
    compare(Compare::Min)
}

fn create_max_function() -> Function {
    compare(Compare::Max)
}

fn compare(compare: Compare) -> Function {
    Function {
        max_args: None,
        min_args: Some(1),
        compiled: Box::new(move |values| {
            let mut prev: Result<Value, Error> = Err(Error::Custom("can't find min value."
                .to_owned()));

            for value in values {
                match value {
                    Value::Array(array) => {
                        for value in array {
                            if prev.is_ok() {
                                if compare == Compare::Min {
                                    if value.lt(prev.as_ref().unwrap())? == to_value(true) {
                                        prev = Ok(value)
                                    }
                                } else if value.gt(prev.as_ref().unwrap())? == to_value(true) {
                                    prev = Ok(value)
                                }
                            } else {
                                prev = Ok(value);
                            }
                        }
                    }
                    _ => {
                        if prev.is_ok() {
                            if compare == Compare::Min {
                                if value.lt(prev.as_ref().unwrap())? == to_value(true) {
                                    prev = Ok(value)
                                }
                            } else if value.gt(prev.as_ref().unwrap())? == to_value(true) {
                                prev = Ok(value)
                            }
                        } else {
                            prev = Ok(value);
                        }
                    }
                }
            }
            prev
        }),
    }
}


fn create_is_empty_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| match *values.first().unwrap() {
            Value::String(ref string) => Ok(to_value(string.is_empty())),
            Value::Array(ref array) => Ok(to_value(array.is_empty())),
            Value::Object(ref object) => Ok(to_value(object.is_empty())),
            Value::Null => Ok(to_value(true)),
            _ => Ok(to_value(false)),
        }),
    }
}

fn create_len_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| {
            let value = values.first().unwrap();
            match *value {
                Value::String(ref string) => Ok(to_value(string.chars().count())),
                Value::Array(ref array) => Ok(to_value(array.len())),
                Value::Object(ref object) => Ok(to_value(object.len())),
                Value::Null => Ok(to_value(0)),
                _ => {
                    Err(Error::Custom(format!("len() only accept string, array, object and \
                                               null. But the given is: {:?}",
                                              value)))
                }
            }
        }),
    }
}

fn create_array_function() -> Function {
    Function::new(|values| Ok(to_value(values)))
}

// ── Math builtins ────────────────────────────────────────────────────────────

fn one_number(name: &'static str, values: &[Value]) -> Result<f64, Error> {
    match values.first().unwrap() {
        Value::Number(n) => Ok(n.as_f64().unwrap()),
        v => Err(Error::Custom(format!("{name}() requires a number, got {v:?}"))),
    }
}

fn create_abs_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| {
            match values.first().unwrap() {
                Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Ok(to_value(i.abs()))
                    } else if let Some(u) = n.as_u64() {
                        Ok(to_value(u))           // u64 is always non-negative
                    } else {
                        Ok(to_value(n.as_f64().unwrap().abs()))
                    }
                }
                v => Err(Error::Custom(format!("abs() requires a number, got {v:?}"))),
            }
        }),
    }
}

fn create_floor_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| {
            one_number("floor", &values).map(|f| to_value(f.floor()))
        }),
    }
}

fn create_ceil_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| {
            one_number("ceil", &values).map(|f| to_value(f.ceil()))
        }),
    }
}

fn create_round_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| {
            one_number("round", &values).map(|f| to_value(f.round()))
        }),
    }
}

fn create_sqrt_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| {
            one_number("sqrt", &values).map(|f| to_value(f.sqrt()))
        }),
    }
}

fn create_pow_function() -> Function {
    Function {
        max_args: Some(2),
        min_args: Some(2),
        compiled: Box::new(|values| match (&values[0], &values[1]) {
            (Value::Number(b), Value::Number(e)) => {
                Ok(to_value(b.as_f64().unwrap().powf(e.as_f64().unwrap())))
            }
            (b, e) => Err(Error::Custom(format!("pow() requires two numbers, got {b:?}, {e:?}"))),
        }),
    }
}

fn create_clamp_function() -> Function {
    Function {
        max_args: Some(3),
        min_args: Some(3),
        compiled: Box::new(|values| match (&values[0], &values[1], &values[2]) {
            (Value::Number(x), Value::Number(lo), Value::Number(hi)) => {
                let x  = x.as_f64().unwrap();
                let lo = lo.as_f64().unwrap();
                let hi = hi.as_f64().unwrap();
                Ok(to_value(x.clamp(lo, hi)))
            }
            (x, lo, hi) => Err(Error::Custom(format!(
                "clamp() requires three numbers, got {x:?}, {lo:?}, {hi:?}"
            ))),
        }),
    }
}

fn create_log_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| one_number("log", &values).map(|f| to_value(f.ln()))),
    }
}

fn create_log2_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| one_number("log2", &values).map(|f| to_value(f.log2()))),
    }
}

fn create_log10_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| one_number("log10", &values).map(|f| to_value(f.log10()))),
    }
}

// ── String builtins ──────────────────────────────────────────────────────────

fn create_contains_function() -> Function {
    Function {
        max_args: Some(2),
        min_args: Some(2),
        compiled: Box::new(|values| match (&values[0], &values[1]) {
            (Value::String(s), Value::String(sub)) => Ok(to_value(s.contains(sub.as_str()))),
            (Value::Array(arr), needle)             => Ok(to_value(arr.contains(needle))),
            (Value::Object(obj), Value::String(key)) => Ok(to_value(obj.contains_key(key.as_str()))),
            (h, n) => Err(Error::Custom(format!("contains() unsupported types: {h:?}, {n:?}"))),
        }),
    }
}

fn create_starts_with_function() -> Function {
    Function {
        max_args: Some(2),
        min_args: Some(2),
        compiled: Box::new(|values| match (&values[0], &values[1]) {
            (Value::String(s), Value::String(prefix)) => Ok(to_value(s.starts_with(prefix.as_str()))),
            (h, n) => Err(Error::Custom(format!("starts_with() requires two strings, got {h:?}, {n:?}"))),
        }),
    }
}

fn create_ends_with_function() -> Function {
    Function {
        max_args: Some(2),
        min_args: Some(2),
        compiled: Box::new(|values| match (&values[0], &values[1]) {
            (Value::String(s), Value::String(suffix)) => Ok(to_value(s.ends_with(suffix.as_str()))),
            (h, n) => Err(Error::Custom(format!("ends_with() requires two strings, got {h:?}, {n:?}"))),
        }),
    }
}

fn create_upper_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| match values.first().unwrap() {
            Value::String(s) => Ok(to_value(s.to_uppercase())),
            v => Err(Error::Custom(format!("upper() requires a string, got {v:?}"))),
        }),
    }
}

fn create_lower_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| match values.first().unwrap() {
            Value::String(s) => Ok(to_value(s.to_lowercase())),
            v => Err(Error::Custom(format!("lower() requires a string, got {v:?}"))),
        }),
    }
}

fn create_trim_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| match values.first().unwrap() {
            Value::String(s) => Ok(to_value(s.trim().to_owned())),
            v => Err(Error::Custom(format!("trim() requires a string, got {v:?}"))),
        }),
    }
}

fn create_replace_function() -> Function {
    Function {
        max_args: Some(3),
        min_args: Some(3),
        compiled: Box::new(|values| match (&values[0], &values[1], &values[2]) {
            (Value::String(s), Value::String(from), Value::String(to)) => {
                Ok(to_value(s.replace(from.as_str(), to.as_str())))
            }
            (s, f, t) => Err(Error::Custom(format!(
                "replace() requires three strings, got {s:?}, {f:?}, {t:?}"
            ))),
        }),
    }
}

fn create_split_function() -> Function {
    Function {
        max_args: Some(2),
        min_args: Some(2),
        compiled: Box::new(|values| match (&values[0], &values[1]) {
            (Value::String(s), Value::String(delim)) => {
                let parts: Vec<Value> = s.split(delim.as_str()).map(|p| to_value(p)).collect();
                Ok(to_value(parts))
            }
            (s, d) => Err(Error::Custom(format!(
                "split() requires two strings, got {s:?}, {d:?}"
            ))),
        }),
    }
}

fn create_join_function() -> Function {
    Function {
        max_args: Some(2),
        min_args: Some(2),
        compiled: Box::new(|values| match (&values[0], &values[1]) {
            (Value::Array(arr), Value::String(delim)) => {
                let parts: Vec<String> = arr.iter().map(value_to_display).collect();
                Ok(to_value(parts.join(delim.as_str())))
            }
            (a, d) => Err(Error::Custom(format!(
                "join() requires an array and a string, got {a:?}, {d:?}"
            ))),
        }),
    }
}

fn create_format_function() -> Function {
    Function {
        max_args: None,
        min_args: Some(1),
        compiled: Box::new(|values| {
            let template = match values.first().unwrap() {
                Value::String(s) => s.clone(),
                v => return Err(Error::Custom(format!("format() first arg must be a string, got {v:?}"))),
            };
            let mut result = template;
            for val in values.iter().skip(1) {
                result = result.replacen("{}", &value_to_display(val), 1);
            }
            Ok(to_value(result))
        }),
    }
}

/// Convert a Value to a human-readable string (used by join/format/str).
fn value_to_display(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() { i.to_string() }
            else if let Some(u) = n.as_u64() { u.to_string() }
            else { n.as_f64().unwrap().to_string() }
        }
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_owned(),
        other => other.to_string(),
    }
}

// ── Type-checking builtins ───────────────────────────────────────────────────

fn create_is_null_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| Ok(to_value(values.first().unwrap().is_null()))),
    }
}

fn create_is_number_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| Ok(to_value(values.first().unwrap().is_number()))),
    }
}

fn create_is_string_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| Ok(to_value(values.first().unwrap().is_string()))),
    }
}

fn create_is_array_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| Ok(to_value(values.first().unwrap().is_array()))),
    }
}

fn create_type_of_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| {
            let t = match values.first().unwrap() {
                Value::Null      => "null",
                Value::Bool(_)   => "bool",
                Value::Number(_) => "number",
                Value::String(_) => "string",
                Value::Array(_)  => "array",
                Value::Object(_) => "object",
            };
            Ok(to_value(t))
        }),
    }
}

// ── Type-conversion builtins ─────────────────────────────────────────────────

fn create_int_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| match values.first().unwrap() {
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(to_value(i))
                } else {
                    Ok(to_value(n.as_u64().unwrap() as i64))
                }
            }
            Value::String(s) => s.trim().parse::<i64>()
                .map(to_value)
                .or_else(|_| s.trim().parse::<f64>().map(|f| to_value(f as i64)))
                .map_err(|_| Error::Custom(format!("int() cannot convert {s:?}"))),
            Value::Bool(b) => Ok(to_value(if *b { 1_i64 } else { 0_i64 })),
            v => Err(Error::Custom(format!("int() unsupported type: {v:?}"))),
        }),
    }
}

fn create_float_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| match values.first().unwrap() {
            Value::Number(n) => Ok(to_value(n.as_f64().unwrap())),
            Value::String(s) => s.trim().parse::<f64>()
                .map(to_value)
                .map_err(|_| Error::Custom(format!("float() cannot convert {s:?}"))),
            Value::Bool(b) => Ok(to_value(if *b { 1.0_f64 } else { 0.0_f64 })),
            v => Err(Error::Custom(format!("float() unsupported type: {v:?}"))),
        }),
    }
}

fn create_str_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| {
            Ok(to_value(value_to_display(values.first().unwrap())))
        }),
    }
}

// ── Object / array inspection ────────────────────────────────────────────────

fn create_keys_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| match values.first().unwrap() {
            Value::Object(obj) => {
                let keys: Vec<Value> = obj.keys().map(|k| to_value(k.as_str())).collect();
                Ok(to_value(keys))
            }
            v => Err(Error::Custom(format!("keys() requires an object, got {v:?}"))),
        }),
    }
}

fn create_values_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| match values.first().unwrap() {
            Value::Object(obj) => {
                let vals: Vec<Value> = obj.values().cloned().collect();
                Ok(to_value(vals))
            }
            v => Err(Error::Custom(format!("values() requires an object, got {v:?}"))),
        }),
    }
}

fn create_index_of_function() -> Function {
    Function {
        max_args: Some(2),
        min_args: Some(2),
        compiled: Box::new(|values| match &values[0] {
            Value::Array(arr) => {
                let needle = &values[1];
                let idx = arr.iter().position(|v| v == needle);
                Ok(to_value(idx.map(|i| i as i64).unwrap_or(-1_i64)))
            }
            v => Err(Error::Custom(format!("index_of() requires an array, got {v:?}"))),
        }),
    }
}

fn create_sort_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| match values.first().unwrap() {
            Value::Array(arr) => {
                let mut sorted = arr.clone();
                sorted.sort_by(|a, b| match (a, b) {
                    (Value::Number(x), Value::Number(y)) => {
                        x.as_f64().partial_cmp(&y.as_f64()).unwrap_or(Ordering::Equal)
                    }
                    (Value::String(x), Value::String(y)) => x.cmp(y),
                    _ => Ordering::Equal,
                });
                Ok(to_value(sorted))
            }
            v => Err(Error::Custom(format!("sort() requires an array, got {v:?}"))),
        }),
    }
}

fn create_reverse_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| match values.first().unwrap() {
            Value::Array(arr) => {
                let mut rev = arr.clone();
                rev.reverse();
                Ok(to_value(rev))
            }
            v => Err(Error::Custom(format!("reverse() requires an array, got {v:?}"))),
        }),
    }
}

fn create_unique_function() -> Function {
    Function {
        max_args: Some(1),
        min_args: Some(1),
        compiled: Box::new(|values| match values.first().unwrap() {
            Value::Array(arr) => {
                let mut seen: Vec<&Value> = Vec::new();
                let unique: Vec<Value> = arr.iter()
                    .filter(|v| {
                        if seen.contains(v) { false }
                        else { seen.push(v); true }
                    })
                    .cloned()
                    .collect();
                Ok(to_value(unique))
            }
            v => Err(Error::Custom(format!("unique() requires an array, got {v:?}"))),
        }),
    }
}

fn create_any_function() -> Function {
    Function {
        max_args: Some(2),
        min_args: Some(2),
        compiled: Box::new(|values| match &values[0] {
            Value::Array(arr) => {
                let needle = &values[1];
                Ok(to_value(arr.iter().any(|v| v == needle)))
            }
            v => Err(Error::Custom(format!("any() requires an array, got {v:?}"))),
        }),
    }
}

fn create_all_function() -> Function {
    Function {
        max_args: Some(2),
        min_args: Some(2),
        compiled: Box::new(|values| match &values[0] {
            Value::Array(arr) => {
                let needle = &values[1];
                Ok(to_value(arr.iter().all(|v| v == needle)))
            }
            v => Err(Error::Custom(format!("all() requires an array, got {v:?}"))),
        }),
    }
}
