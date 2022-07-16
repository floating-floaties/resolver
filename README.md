resolver
====


Source
---

> This work is a derivetive of this repository: https://github.com/fengcen/eval
>
> The aforementioned repository has been abandoned, hence the reason for this repository/crate.
>

<!-- [![Project Status: Abandoned – Initial development has started, but there has not yet been a stable, usable release; the project has been abandoned and the author(s) do not intend on continuing development.](https://www.repostatus.org/badges/latest/abandoned.svg)](https://www.repostatus.org/#abandoned) -->

<!-- [![docs](https://docs.rs/eval/badge.svg?version=0.4.3 "docs")](https://docs.rs/eval) -->

<!-- Eval is a powerful expression evaluator. -->

<!-- [Document](https://docs.rs/eval) -->
--------------------------------

Features
--------

Supported operators: `!` `!=` `""` `''` `()` `[]` `,` `>` `<` `>=` `<=` `==`
`+` `-` `*` `/` `%` `&&` `||` `n..m`.

Built-in functions: `min()` `max()` `len()` `is_empty()` `array()`.

Where can eval be used?
-----------------------

* Template engine
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

License
-------

resolver is under the terms of the [GPL-3.0-only](LICENSE) license.

See [LICENSE](LICENSE) for details.
