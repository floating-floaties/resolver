resolver
====


Source
---

> This work is a derivative of this repository: https://github.com/fengcen/eval
>
> The aforementioned repository has been abandoned, hence the reason for this repository/crate.
>

--------------------------------

Features
--------

### Operators

| Operator | Priority | Description | Example |
|----------|----------|-------------|---------|
| `**` | 12 | Exponentiation (left-assoc) | `2 ** 10` |
| `-x` `~x` | 11 | Unary negation / bitwise NOT | `-5`, `~0xFF` |
| `*` `/` `%` | 10 | Multiplicative | `10 % 3` |
| `<<` `>>` | 9 | Bit shifts | `1 << 4` |
| `+` `-` | 8 | Additive | `2 + 3` |
| `&` | 7 | Bitwise AND | `12 & 10` |
| `==` `!=` `>` `<` `>=` `<=` `in` `not in` | 6 | Comparison / membership | `x in arr` |
| `^` | 5 | Bitwise XOR | `12 ^ 10` |
| `&&` | 4 | Logical AND (short-circuit) | `a && b` |
| `\|` | 3 | Bitwise OR | `12 \| 10` |
| `\|\|` | 2 | Logical OR (short-circuit) | `a \|\| b` |
| `??` | 1 | Null-coalescing | `value ?? "default"` |
| `!` | — | Logical NOT (prefix) | `!flag` |
| `n..m` | — | Integer range | `0..5` |
| `()` `[]` `.` | — | Grouping, indexing, field access | `obj.foo[0]` |
| `""` `''` | — | String literals | `"hello"` |

### Built-in Functions

**Conditional:**

| Function | Description |
|----------|-------------|
| `if(cond, then, else)` | Short-circuit conditional — only evaluates the matching branch |

**Collections:**

| Function | Description |
|----------|-------------|
| `min(a, b, ...)` | Minimum value |
| `max(a, b, ...)` | Maximum value |
| `len(x)` | Length of string, array, or object |
| `is_empty(x)` | True if string/array/object is empty or value is null |
| `array(a, b, ...)` | Create an array |
| `sort(arr)` | Sort array of numbers or strings (ascending) |
| `reverse(arr)` | Reverse an array |
| `unique(arr)` | Remove duplicate elements (preserves first occurrence) |
| `index_of(arr, val)` | Index of first match, or `-1` if not found |
| `any(arr, val)` | True if any element equals `val` |
| `all(arr, val)` | True if every element equals `val` |
| `keys(obj)` | Array of object keys |
| `values(obj)` | Array of object values |

**Math:**

| Function | Description |
|----------|-------------|
| `abs(x)` | Absolute value |
| `floor(x)` | Floor (round down) |
| `ceil(x)` | Ceiling (round up) |
| `round(x)` | Round to nearest integer (half-away-from-zero) |
| `sqrt(x)` | Square root |
| `pow(base, exp)` | Exponentiation (also available as `**`) |
| `clamp(x, min, max)` | Clamp `x` to `[min, max]` |
| `log(x)` | Natural logarithm (ln) |
| `log2(x)` | Base-2 logarithm |
| `log10(x)` | Base-10 logarithm |

**String:**

| Function | Description |
|----------|-------------|
| `contains(haystack, needle)` | Check if string/array/object contains value |
| `starts_with(s, prefix)` | Check if string starts with prefix |
| `ends_with(s, suffix)` | Check if string ends with suffix |
| `upper(s)` | Convert to uppercase |
| `lower(s)` | Convert to lowercase |
| `trim(s)` | Trim leading/trailing whitespace |
| `replace(s, from, to)` | Replace all occurrences of `from` with `to` |
| `split(s, delim)` | Split string into an array |
| `join(arr, delim)` | Join array elements into a string |
| `format(tmpl, ...)` | Replace `{}` placeholders with arguments in order |

**Type checking:**

| Function | Description |
|----------|-------------|
| `is_null(x)` | Check if value is null |
| `is_number(x)` | Check if value is a number |
| `is_string(x)` | Check if value is a string |
| `is_array(x)` | Check if value is an array |
| `type_of(x)` | Returns `"null"`, `"bool"`, `"number"`, `"string"`, `"array"`, or `"object"` |

**Type conversion:**

| Function | Description |
|----------|-------------|
| `int(x)` | Convert to integer (string, number, or bool) |
| `float(x)` | Convert to float (string, number, or bool) |
| `str(x)` | Convert to string |

Where can resolver be used?
-----------------------

* Template engine
* Configuration with computed values
* User-defined filters / rules
* ...

Usage
-----

Add dependency to Cargo.toml

```toml
[dependencies]
resolver = "^0.1"
```

In your `main.rs` or `lib.rs`:

```rust
extern crate resolver;
```

Examples
--------

You can do mathematical calculations with supported operators:

```rust
use resolver::{eval, to_value};

assert_eq!(eval("1 + 2 + 3"), Ok(to_value(6)));
assert_eq!(eval("2 * 2 + 3"), Ok(to_value(7)));
assert_eq!(eval("2 / 2 + 3"), Ok(to_value(4.0)));
assert_eq!(eval("2 / 2 + 3 / 3"), Ok(to_value(2.0)));
```

You can eval with context:

```rust
use resolver::{Expr, to_value};

assert_eq!(Expr::new("foo == bar")
               .value("foo", true)
               .value("bar", true)
               .exec(),
           Ok(to_value(true)));
```

You can access data like javascript by using `.` and `[]`. `[]` supports expression.

```rust
use resolver::{Expr, to_value};
use std::collections::HashMap;

let mut object = HashMap::new();
object.insert("foos", vec!["Hello", "world", "!"]);

assert_eq!(Expr::new("object.foos[1-1] == 'Hello'")
               .value("object", object)
               .exec(),
           Ok(to_value(true)));
```

You can eval with function:

```rust
use resolver::{Expr, to_value};

assert_eq!(Expr::new("say_hello()")
               .function("say_hello", |_| Ok(to_value("Hello world!")))
               .exec(),
           Ok(to_value("Hello world!")));
```

You can create an array with `array()`:

```rust
use resolver::{eval, to_value};

assert_eq!(eval("array(1, 2, 3, 4, 5)"), Ok(to_value(vec![1, 2, 3, 4, 5])));
```

You can create an integer array with `n..m`:

```rust
use resolver::{eval, to_value};

assert_eq!(eval("0..5"), Ok(to_value(vec![0, 1, 2, 3, 4])));
```

Null-coalescing lets you provide defaults for missing values:

```rust
use resolver::{Expr, to_value};

assert_eq!(Expr::new("val ?? 'fallback'")
               .exec(),
           Ok(to_value("fallback")));
```

The `in` / `not in` operators check membership in arrays, objects, or strings:

```rust
use resolver::{eval, to_value};

assert_eq!(eval("2 in array(1, 2, 3)"),     Ok(to_value(true)));
assert_eq!(eval("5 not in array(1, 2, 3)"), Ok(to_value(true)));
assert_eq!(eval("'lo' in 'hello'"),          Ok(to_value(true)));
```

Use `if()` for conditional expressions (only the matching branch is evaluated):

```rust
use resolver::{eval, to_value};

assert_eq!(eval("if(3 > 2, 'yes', 'no')"), Ok(to_value("yes")));
```

Bitwise operators and shifts work on integers:

```rust
use resolver::{eval, to_value};

assert_eq!(eval("0xFF & 0x0F"), Ok(to_value(15_i64)));
assert_eq!(eval("1 << 8"),      Ok(to_value(256_i64)));
assert_eq!(eval("~0"),          Ok(to_value(-1_i64)));
```

String utilities:

```rust
use resolver::{eval, to_value};

assert_eq!(eval("split('a,b,c', ',')"),            Ok(to_value(vec!["a", "b", "c"])));
assert_eq!(eval("join(array('x', 'y'), '-')"),     Ok(to_value("x-y")));
assert_eq!(eval("replace('hello', 'l', 'r')"),     Ok(to_value("herro")));
assert_eq!(eval("format('Hi, {}!', 'world')"),     Ok(to_value("Hi, world!")));
```

Array utilities:

```rust
use resolver::{eval, to_value};

assert_eq!(eval("sort(array(3, 1, 2))"),              Ok(to_value(vec![1_i64, 2, 3])));
assert_eq!(eval("unique(array(1, 2, 1))"),            Ok(to_value(vec![1_i64, 2])));
assert_eq!(eval("index_of(array(10, 20, 30), 20)"),  Ok(to_value(1_i64)));
assert_eq!(eval("any(array(1, 2, 3), 2)"),           Ok(to_value(true)));
```

License
-------

resolver is under the terms of the [MIT](LICENSE) license.

See [LICENSE](LICENSE) for details.
