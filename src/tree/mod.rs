use crate::builtin::BuiltIn;
use crate::error::Error;
use crate::math::Math;
use crate::node::Node;
use crate::operator::Operator;
use crate::Compiled;
use crate::{to_value, ConstFunctions};
use crate::{Context, Functions};
use serde_json::Value;
use std::cell::RefCell;
use std::clone::Clone;
use std::rc::Rc;
use std::str::FromStr;

#[derive(Default)]
pub struct Tree {
    pub raw: String,
    pub pos: Vec<usize>,
    pub operators: Vec<Operator>,
    pub node: Option<Node>,
}

impl Tree {
    pub fn new<T: Into<String>>(raw: T) -> Tree {
        Tree {
            raw: raw.into(),
            ..Default::default()
        }
    }

    pub fn parse_pos(&mut self) -> Result<(), Error> {
        let mut found_quote = false;
        let mut pos = Vec::new();

        for (index, cur) in self.raw.char_indices() {
            match cur {
                '(' | ')' | '+' | '-' | '*' | '/' | ',' | ' ' | '!' | '=' | '>' | '<' | '\''
                | '[' | ']' | '.' | '%' | '&' | '|' | '?' | '^' | '~' => {
                    if !found_quote {
                        pos.push(index);
                        pos.push(index + 1);
                    }
                }
                '"' => {
                    found_quote = !found_quote;
                    pos.push(index);
                    pos.push(index + 1);
                }
                _ => (),
            }
        }

        pos.push(self.raw.len());

        self.pos = pos;
        Ok(())
    }

    pub fn parse_operators(&mut self) -> Result<(), Error> {
        let mut operators = Vec::new();
        let mut start;
        let mut end = 0;
        let mut parenthesis = 0;
        let mut quote = None;
        let mut prev = String::new();
        let mut number = String::new();

        for pos_ref in &self.pos {
            let pos = *pos_ref;
            if pos == 0 {
                continue;
            } else {
                start = end;
                end = pos;
            }

            let raw = self.raw[start..end].to_owned();

            if raw.is_empty() {
                continue;
            }

            let operator = Operator::from_str(&raw).unwrap();
            match operator {
                Operator::DoubleQuotes | Operator::SingleQuote => {
                    if quote.is_some() {
                        if quote.as_ref() == Some(&operator) {
                            operators.push(Operator::Value(to_value(&prev)));
                            prev.clear();
                            quote = None;
                            continue;
                        }
                    } else {
                        quote = Some(operator);
                        prev.clear();
                        continue;
                    }
                }
                _ => (),
            };

            if quote.is_some() {
                prev += &raw;
                continue;
            }

            if parse_number(&raw).is_some() || operator.is_dot() {
                number += &raw;
                continue;
            } else if !number.is_empty() {
                operators.push(Operator::from_str(&number).unwrap());
                number.clear();
                // Clear prev so a trailing * or ? doesn't falsely combine with
                // the operator that follows the number (e.g. `2 * 3 ** 4`).
                prev.clear();
            }

            if raw == "=" {
                if prev == "!" || prev == ">" || prev == "<" || prev == "=" {
                    prev.push('=');
                    operators.push(Operator::from_str(&prev).unwrap());
                    prev.clear();
                } else {
                    prev = raw;
                }
                continue;
            } else if raw == "!" || raw == ">" || raw == "<" {
                if prev == raw && (raw == "<" || raw == ">") {
                    // << or >>
                    operators.push(if raw == "<" { Operator::Shl(9) } else { Operator::Shr(9) });
                    prev.clear();
                } else {
                    if prev == "!" || prev == ">" || prev == "<" {
                        operators.push(Operator::from_str(&prev).unwrap());
                    } else if prev == "&" {
                        operators.push(Operator::BitAnd(7));
                    } else if prev == "|" {
                        operators.push(Operator::BitOr(3));
                    }
                    prev = raw;
                }
                continue;
            } else if prev == "!" || prev == ">" || prev == "<" {
                operators.push(Operator::from_str(&prev).unwrap());
                prev.clear();
            }

            if raw == "&" || raw == "|" {
                if prev == raw {
                    // && or ||
                    let mut combined = prev.clone();
                    combined.push_str(&raw);
                    operators.push(Operator::from_str(&combined).unwrap());
                    prev.clear();
                } else {
                    // Flush any pending single & or | as bitwise operators
                    if prev == "&" { operators.push(Operator::BitAnd(7)); }
                    else if prev == "|" { operators.push(Operator::BitOr(3)); }
                    prev = raw;
                }
                continue;
            }
            // Flush pending single & or | before any other operator
            if prev == "&" { operators.push(Operator::BitAnd(7)); prev.clear(); }
            else if prev == "|" { operators.push(Operator::BitOr(3)); prev.clear(); }

            match operator {
                Operator::LeftParenthesis => {
                    parenthesis += 1;

                    if !operators.is_empty() {
                        let prev_operator = operators.pop().unwrap();
                        if prev_operator.is_identifier() {
                            operators.push(Operator::Function(
                                prev_operator.get_identifier().to_owned(),
                            ));
                            operators.push(operator);
                            continue;
                        } else {
                            operators.push(prev_operator);
                        }
                    }
                }
                Operator::RightParenthesis => parenthesis -= 1,
                Operator::WhiteSpace => continue,
                _ => (),
            }

            // Detect ** (exponentiation): second * seen right after a first *.
            // The first * was already pushed as Mul; replace it with Pow.
            if raw == "*" && prev == "*" {
                if operators.last().map_or(false, |op| matches!(op, Operator::Mul(_))) {
                    operators.pop();
                }
                operators.push(Operator::Pow(12));
                prev.clear();
                continue;
            }

            // Detect ?? (null-coalesce).
            if raw == "?" {
                if prev == "?" {
                    operators.push(Operator::NullCoalesce(1));
                    prev.clear();
                } else {
                    prev = raw;
                }
                continue;
            } else if prev == "?" {
                return Err(Error::UnsupportedOperator("?".to_string()));
            }

            // ~ is always a unary bitwise NOT; push directly.
            if let Operator::BitNot = operator {
                operators.push(Operator::BitNot);
                prev = raw;
                continue;
            }

            // Detect unary minus: `-` that follows no value/closing-bracket.
            if let Operator::Sub(_) = operator {
                let is_unary = operators.last().map_or(true, |last| {
                    !matches!(last,
                        Operator::Value(_) |
                        Operator::Identifier(_) |
                        Operator::RightParenthesis |
                        Operator::RightSquareBracket)
                });
                if is_unary {
                    operators.push(Operator::UnaryMinus(11));
                    prev = raw;
                    continue;
                }
            }

            // `not in` → NotIn operator: when `in` immediately follows `not` identifier.
            if let Operator::In(_) = operator {
                if matches!(operators.last(), Some(Operator::Identifier(s)) if s == "not") {
                    operators.pop();
                    operators.push(Operator::NotIn(6));
                    prev = raw;
                    continue;
                }
            }

            prev = raw;
            operators.push(operator);
        }

        if !number.is_empty() {
            operators.push(Operator::from_str(&number).unwrap());
        }

        if parenthesis != 0 {
            Err(Error::UnpairedBrackets)
        } else {
            self.operators = operators;
            Ok(())
        }
    }

    pub fn parse_node(&mut self) -> Result<(), Error> {
        let mut parsing_nodes = Vec::<Node>::new();

        for operator in &self.operators {
            match *operator {
                Operator::Add(priority)
                | Operator::Sub(priority)
                | Operator::Mul(priority)
                | Operator::Div(priority)
                | Operator::Not(priority)
                | Operator::Eq(priority)
                | Operator::Ne(priority)
                | Operator::Gt(priority)
                | Operator::Lt(priority)
                | Operator::Ge(priority)
                | Operator::And(priority)
                | Operator::Or(priority)
                | Operator::Le(priority)
                | Operator::Dot(priority)
                | Operator::LeftSquareBracket(priority)
                | Operator::Rem(priority)
                | Operator::Pow(priority)
                | Operator::NullCoalesce(priority)
                | Operator::In(priority)
                | Operator::NotIn(priority)
                | Operator::BitAnd(priority)
                | Operator::BitOr(priority)
                | Operator::BitXor(priority)
                | Operator::Shl(priority)
                | Operator::Shr(priority) => {
                    if !parsing_nodes.is_empty() {
                        let mut prev = parsing_nodes.pop().unwrap();
                        if prev.is_value_or_full_children() {
                            if prev.operator.get_priority() < priority && !prev.closed {
                                parsing_nodes.extend_from_slice(&rob_to(prev, operator.to_node()));
                            } else {
                                // Fold prev into any unclosed parent with >= priority
                                // so that e.g. `1 in array(1,2,3) && ...` correctly
                                // makes the function result a child of `in` before `&&`
                                // consumes it.
                                loop {
                                    if let Some(parent) = parsing_nodes.last() {
                                        if parent.is_unclosed_arithmetic()
                                            && parent.operator.get_priority() >= priority
                                            && parent.operator.get_max_args().map_or(false, |n| n > 0)
                                        {
                                            let mut parent = parsing_nodes.pop().unwrap();
                                            parent.add_child(prev);
                                            if parent.is_enough() {
                                                parent.closed = true;
                                            }
                                            prev = parent;
                                        } else {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                }
                                parsing_nodes.push(operator.children_to_node(vec![prev]));
                            }
                        } else if prev.operator.can_at_beginning() {
                            parsing_nodes.push(prev);
                            parsing_nodes.push(operator.to_node());
                        } else {
                            return Err(Error::DuplicateOperatorNode);
                        }
                    } else if operator.can_at_beginning() {
                        parsing_nodes.push(operator.to_node());
                    } else {
                        return Err(Error::StartWithNonValueOperator);
                    }
                }
                // UnaryMinus is pushed directly (like a left-paren) so it
                // can appear after any binary operator without triggering the
                // DuplicateOperatorNode error.
                Operator::Function(_) | Operator::LeftParenthesis | Operator::UnaryMinus(_) | Operator::BitNot => {
                    parsing_nodes.push(operator.to_node())
                }
                Operator::Comma => close_comma(&mut parsing_nodes)?,
                Operator::RightParenthesis | Operator::RightSquareBracket => {
                    close_bracket(&mut parsing_nodes, operator.get_left())?
                }
                Operator::Value(_) | Operator::Identifier(_) => {
                    append_value_to_last_node(&mut parsing_nodes, operator)?
                }
                _ => (),
            }
        }

        self.node = Some(get_final_node(parsing_nodes)?);
        Ok(())
    }

    pub fn compile(mut self) -> Result<Compiled, Error> {
        self.parse_pos()?;
        self.parse_operators()?;
        self.parse_node()?;
        let node = self.node.unwrap();
        let builtin = BuiltIn::create_builtins();

        Ok(Box::new(
            move |contexts, functions, const_functions| -> Result<Value, Error> {
                return exec_node(&node, &builtin, contexts, functions, const_functions);

            #[rustfmt::skip]
            fn exec_node(node: &Node,
                         builtin: &Functions,
                         contexts: &[Context],
                         functions: &Functions,
                         const_functions: Rc<RefCell<ConstFunctions>>,)
                         -> Result<Value, Error> {
                match node.operator {
                    Operator::Add(_) => {
                        exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))
                            ?
                            .add(&exec_node(&node.get_last_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?)
                    }
                    Operator::Mul(_) => {
                        exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))
                            ?
                            .mul(&exec_node(&node.get_last_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?)
                    }
                    Operator::Sub(_) => {
                        exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))
                            ?
                            .sub(&exec_node(&node.get_last_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?)
                    }
                    Operator::Div(_) => {
                        exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))
                            ?
                            .div(&exec_node(&node.get_last_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?)
                    }
                    Operator::Rem(_) => {
                        exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))
                            ?
                            .rem(&exec_node(&node.get_last_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?)
                    }
                    Operator::Eq(_) => {
                        Math::eq(&exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?,
                                 &exec_node(&node.get_last_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?)
                    }
                    Operator::Ne(_) => {
                        Math::ne(&exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?,
                                 &exec_node(&node.get_last_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?)
                    }
                    Operator::Gt(_) => {
                        exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))
                            ?
                            .gt(&exec_node(&node.get_last_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?)
                    }
                    Operator::Lt(_) => {
                        exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))
                            ?
                            .lt(&exec_node(&node.get_last_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?)
                    }
                    Operator::Ge(_) => {
                        exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))
                            ?
                            .ge(&exec_node(&node.get_last_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?)
                    }
                    Operator::Le(_) => {
                        exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))
                            ?
                            .le(&exec_node(&node.get_last_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?)
                    }
                    Operator::And(_) => {
                        let left = exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?;
                        match left {
                            Value::Bool(false) => Ok(Value::Bool(false)),
                            Value::Bool(true) => {
                                let right = exec_node(&node.get_last_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?;
                                match right {
                                    Value::Bool(b) => Ok(Value::Bool(b)),
                                    _ => Err(Error::UnsupportedTypes(format!("{:?}", left), format!("{:?}", right))),
                                }
                            }
                            _ => Err(Error::UnsupportedTypes(format!("{:?}", left), "bool".to_string())),
                        }
                    }
                    Operator::Or(_) => {
                        let left = exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?;
                        match left {
                            Value::Bool(true) => Ok(Value::Bool(true)),
                            Value::Bool(false) => {
                                let right = exec_node(&node.get_last_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?;
                                match right {
                                    Value::Bool(b) => Ok(Value::Bool(b)),
                                    _ => Err(Error::UnsupportedTypes(format!("{:?}", left), format!("{:?}", right))),
                                }
                            }
                            _ => Err(Error::UnsupportedTypes(format!("{:?}", left), "bool".to_string())),
                        }
                    }
                    Operator::UnaryMinus(_) => {
                        let value = exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?;
                        match value {
                            Value::Number(ref n) => {
                                if let Some(i) = n.as_i64() {
                                    Ok(to_value(-i))
                                } else if let Some(f) = n.as_f64() {
                                    Ok(to_value(-f))
                                } else {
                                    Ok(to_value(-(n.as_u64().unwrap() as i64)))
                                }
                            }
                            _ => Err(Error::ExpectedNumber),
                        }
                    }
                    Operator::Pow(_) => {
                        let base = exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?;
                        let exp  = exec_node(&node.get_last_child()?,  builtin, contexts, functions, Rc::clone(&const_functions))?;
                        match (&base, &exp) {
                            (Value::Number(b), Value::Number(e)) => {
                                Ok(to_value(b.as_f64().unwrap().powf(e.as_f64().unwrap())))
                            }
                            _ => Err(Error::UnsupportedTypes(format!("{:?}", base), format!("{:?}", exp))),
                        }
                    }
                    Operator::NullCoalesce(_) => {
                        let left = exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?;
                        match left {
                            Value::Null => exec_node(&node.get_last_child()?, builtin, contexts, functions, Rc::clone(&const_functions)),
                            _ => Ok(left),
                        }
                    }
                    Operator::In(_) => {
                        let left  = exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?;
                        let right = exec_node(&node.get_last_child()?,  builtin, contexts, functions, Rc::clone(&const_functions))?;
                        match &right {
                            Value::Array(arr) => Ok(to_value(arr.contains(&left))),
                            Value::Object(obj) => {
                                if let Some(key) = left.as_str() {
                                    Ok(to_value(obj.contains_key(key)))
                                } else {
                                    Err(Error::Custom(format!(
                                        "in: left operand must be a string when right is an object, got {:?}", left
                                    )))
                                }
                            }
                            Value::String(s) => {
                                if let Some(sub) = left.as_str() {
                                    Ok(to_value(s.contains(sub)))
                                } else {
                                    Err(Error::Custom(format!(
                                        "in: left operand must be a string when right is a string, got {:?}", left
                                    )))
                                }
                            }
                            _ => Err(Error::UnsupportedTypes(format!("{:?}", left), format!("{:?}", right))),
                        }
                    }
                    Operator::NotIn(_) => {
                        let left  = exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?;
                        let right = exec_node(&node.get_last_child()?,  builtin, contexts, functions, Rc::clone(&const_functions))?;
                        match &right {
                            Value::Array(arr) => Ok(to_value(!arr.contains(&left))),
                            Value::Object(obj) => {
                                if let Some(key) = left.as_str() {
                                    Ok(to_value(!obj.contains_key(key)))
                                } else {
                                    Err(Error::Custom(format!(
                                        "not in: left operand must be a string when right is an object, got {:?}", left
                                    )))
                                }
                            }
                            Value::String(s) => {
                                if let Some(sub) = left.as_str() {
                                    Ok(to_value(!s.contains(sub)))
                                } else {
                                    Err(Error::Custom(format!(
                                        "not in: left operand must be a string when right is a string, got {:?}", left
                                    )))
                                }
                            }
                            _ => Err(Error::UnsupportedTypes(format!("{:?}", left), format!("{:?}", right))),
                        }
                    }
                    Operator::BitAnd(_) => {
                        let left  = exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?;
                        let right = exec_node(&node.get_last_child()?,  builtin, contexts, functions, Rc::clone(&const_functions))?;
                        match (&left, &right) {
                            (Value::Number(l), Value::Number(r)) => {
                                let l = l.as_i64().ok_or(Error::ExpectedNumber)?;
                                let r = r.as_i64().ok_or(Error::ExpectedNumber)?;
                                Ok(to_value(l & r))
                            }
                            _ => Err(Error::UnsupportedTypes(format!("{:?}", left), format!("{:?}", right))),
                        }
                    }
                    Operator::BitOr(_) => {
                        let left  = exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?;
                        let right = exec_node(&node.get_last_child()?,  builtin, contexts, functions, Rc::clone(&const_functions))?;
                        match (&left, &right) {
                            (Value::Number(l), Value::Number(r)) => {
                                let l = l.as_i64().ok_or(Error::ExpectedNumber)?;
                                let r = r.as_i64().ok_or(Error::ExpectedNumber)?;
                                Ok(to_value(l | r))
                            }
                            _ => Err(Error::UnsupportedTypes(format!("{:?}", left), format!("{:?}", right))),
                        }
                    }
                    Operator::BitXor(_) => {
                        let left  = exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?;
                        let right = exec_node(&node.get_last_child()?,  builtin, contexts, functions, Rc::clone(&const_functions))?;
                        match (&left, &right) {
                            (Value::Number(l), Value::Number(r)) => {
                                let l = l.as_i64().ok_or(Error::ExpectedNumber)?;
                                let r = r.as_i64().ok_or(Error::ExpectedNumber)?;
                                Ok(to_value(l ^ r))
                            }
                            _ => Err(Error::UnsupportedTypes(format!("{:?}", left), format!("{:?}", right))),
                        }
                    }
                    Operator::BitNot => {
                        let value = exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?;
                        match &value {
                            Value::Number(n) => {
                                let i = n.as_i64().ok_or(Error::ExpectedNumber)?;
                                Ok(to_value(!i))
                            }
                            _ => Err(Error::ExpectedNumber),
                        }
                    }
                    Operator::Shl(_) => {
                        let left  = exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?;
                        let right = exec_node(&node.get_last_child()?,  builtin, contexts, functions, Rc::clone(&const_functions))?;
                        match (&left, &right) {
                            (Value::Number(l), Value::Number(r)) => {
                                let l = l.as_i64().ok_or(Error::ExpectedNumber)?;
                                let r = r.as_u64().ok_or(Error::ExpectedNumber)?;
                                if r >= 64 { return Err(Error::Custom("shift amount too large".into())); }
                                Ok(to_value(l << r))
                            }
                            _ => Err(Error::UnsupportedTypes(format!("{:?}", left), format!("{:?}", right))),
                        }
                    }
                    Operator::Shr(_) => {
                        let left  = exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?;
                        let right = exec_node(&node.get_last_child()?,  builtin, contexts, functions, Rc::clone(&const_functions))?;
                        match (&left, &right) {
                            (Value::Number(l), Value::Number(r)) => {
                                let l = l.as_i64().ok_or(Error::ExpectedNumber)?;
                                let r = r.as_u64().ok_or(Error::ExpectedNumber)?;
                                if r >= 64 { return Err(Error::Custom("shift amount too large".into())); }
                                Ok(to_value(l >> r))
                            }
                            _ => Err(Error::UnsupportedTypes(format!("{:?}", left), format!("{:?}", right))),
                        }
                    }
                    // `if(cond, then, else)` — short-circuit: only evaluates the matching branch.
                    Operator::Function(ref ident) if ident == "if" => {
                        if node.children.len() != 3 {
                            return if node.children.len() < 3 {
                                Err(Error::ArgumentsLess(3))
                            } else {
                                Err(Error::ArgumentsGreater(3))
                            };
                        }
                        let cond = exec_node(&node.children[0], builtin, contexts, functions, Rc::clone(&const_functions))?;
                        match cond {
                            Value::Bool(true)  => exec_node(&node.children[1], builtin, contexts, functions, Rc::clone(&const_functions)),
                            Value::Bool(false) => exec_node(&node.children[2], builtin, contexts, functions, Rc::clone(&const_functions)),
                            _ => Err(Error::ExpectedBoolean(cond)),
                        }
                    }
                    Operator::Function(ref ident) => {
                        let mut values = Vec::new();
                        for node in &node.children {
                            values.push(exec_node(node, builtin, contexts, functions, Rc::clone(&const_functions))?);
                        }

                        if let Some(fo) = functions.get(ident) {
                            node.check_function_args(fo)?;
                            (fo.compiled)(values)
                        } else if let Some(f) = const_functions.borrow().get(ident) {
                            (f.compiled)(values)
                        } else if let Some(fo) = builtin.get(ident) {
                            node.check_function_args(fo)?;
                            (fo.compiled)(values)
                        } else {
                            Err(Error::FunctionNotExists(ident.to_owned()))
                        }
                    }
                    Operator::Value(ref value) => Ok(value.clone()),
                    Operator::Not(_) => {
                        let value =
                            exec_node(&node.get_first_child()?, builtin, contexts, functions, Rc::clone(&const_functions))?;
                        match value {
                            Value::Bool(boolean) => Ok(Value::Bool(!boolean)),
                            Value::Null => Ok(Value::Bool(true)),
                            _ => Err(Error::ExpectedBoolean(value)),
                        }
                    }
                    Operator::Dot(_) => {
                        let mut value = None;
                        for child in &node.children {
                            if value.is_none() {
                                let name = exec_node(child, builtin, contexts, functions, Rc::clone(&const_functions))?;
                                if name.is_string() {
                                    value = find(contexts, name.as_str().unwrap());
                                    if value.is_none() {
                                        return Ok(Value::Null);
                                    }
                                } else if name.is_object() {
                                    value = Some(name);
                                } else if name.is_null() {
                                    return Ok(Value::Null);
                                } else {
                                    return Err(Error::ExpectedObject);
                                }
                            } else if child.operator.is_identifier() {
                                value = value.as_ref()
                                    .unwrap()
                                    .get(child.operator.get_identifier())
                                    .cloned();
                            } else {
                                return Err(Error::ExpectedIdentifier);
                            }
                        }

                        if let Some(v) = value {
                            Ok(v)
                        } else {
                            Ok(Value::Null)
                        }
                    }
                    Operator::LeftSquareBracket(_) => {
                        let mut value = None;
                        for child in &node.children {
                            let name = exec_node(child, builtin, contexts, functions, Rc::clone(&const_functions))?;
                            if value.is_none() {
                                if name.is_string() {
                                    value = find(contexts, name.as_str().unwrap());
                                    if value.is_none() {
                                        return Ok(Value::Null);
                                    }
                                } else if name.is_array() || name.is_object(){
                                    value = Some(name);
                                } else if name.is_null() {
                                    return Ok(Value::Null);
                                } else {
                                    return Err(Error::ExpectedArray);
                                }
                            } else if value.as_ref().unwrap().is_object() {
                                if name.is_string() {
                                    value = value.as_ref()
                                        .unwrap()
                                        .get(name.as_str().unwrap())
                                        .cloned();
                                } else {
                                    return Err(Error::ExpectedIdentifier);
                                }
                            } else if name.is_u64() {
                                if value.as_ref().unwrap().is_array() {
                                    let raw_idx = name.as_u64().unwrap();
                                    let idx = usize::try_from(raw_idx).map_err(|_| {
                                        Error::Custom(format!("array index {} is out of range", raw_idx))
                                    })?;
                                    value = value.as_ref()
                                        .unwrap()
                                        .as_array()
                                        .unwrap()
                                        .get(idx)
                                        .cloned();
                                } else {
                                    return Err(Error::ExpectedArray);
                                }
                            } else if name.is_i64() {
                                return Err(Error::Custom(format!(
                                    "array index must be non-negative, got {}",
                                    name.as_i64().unwrap()
                                )));
                            } else {
                                return Err(Error::ExpectedNumber);
                            }
                        }
                        if let Some(v) = value {
                            Ok(v)
                        } else {
                            Ok(Value::Null)
                        }
                    }
                    Operator::Identifier(ref ident) => {
                        let number = parse_number(ident);
                        if let Some(n) = number {
                            Ok(n)
                        } else if is_range(ident) {
                            parse_range(ident)
                        } else {
                            match find(contexts, ident) {
                                Some(value) => Ok(value),
                                None => Ok(Value::Null),
                            }
                        }
                    }
                    _ => Err(Error::CanNotExec(node.operator.clone())),
                }
            }
            },
        ))
    }
}

fn append_value_to_last_node(
    parsing_nodes: &mut Vec<Node>,
    operator: &Operator,
) -> Result<(), Error> {
    let mut node = operator.to_node();
    node.closed = true;

    if let Some(mut prev) = parsing_nodes.pop() {
        if prev.is_dot() {
            prev.add_child(node);
            prev.closed = true;
            parsing_nodes.push(prev);
        } else if prev.is_left_square_bracket() {
            parsing_nodes.push(prev);
            parsing_nodes.push(node);
        } else if prev.is_value_or_full_children() {
            return Err(Error::DuplicateValueNode);
        } else if prev.is_enough() {
            parsing_nodes.push(prev);
            parsing_nodes.push(node);
        } else if prev.operator.can_have_child() {
            prev.add_child(node);
            parsing_nodes.push(prev);
        } else {
            return Err(Error::CanNotAddChild);
        }
    } else {
        parsing_nodes.push(node);
    }

    Ok(())
}

fn get_final_node(mut parsing_nodes: Vec<Node>) -> Result<Node, Error> {
    if parsing_nodes.is_empty() {
        return Err(Error::NoFinalNode);
    }

    while parsing_nodes.len() != 1 {
        let last = parsing_nodes.pop().unwrap();
        let mut prev = parsing_nodes.pop().unwrap();
        if prev.operator.can_have_child() {
            prev.add_child(last);
            parsing_nodes.push(prev);
        } else {
            return Err(Error::CanNotAddChild);
        }
    }

    Ok(parsing_nodes.pop().unwrap())
}

fn close_bracket(parsing_nodes: &mut Vec<Node>, bracket: Operator) -> Result<(), Error> {
    loop {
        if parsing_nodes.len() < 2 {
            return Err(Error::UnpairedBrackets);
        }
        let mut current = parsing_nodes.pop().unwrap();
        let mut prev = parsing_nodes.pop().unwrap();

        if current.operator.is_left_square_bracket() {
            return Err(Error::BracketNotWithFunction);
        } else if prev.operator.is_left_square_bracket() {
            prev.add_child(current);
            prev.closed = true;
            parsing_nodes.push(prev);
            break;
        } else if current.operator == bracket {
            if prev.is_unclosed_function() {
                prev.closed = true;
                parsing_nodes.push(prev);
                break;
            } else {
                return Err(Error::BracketNotWithFunction);
            }
        } else if prev.operator == bracket {
            if !current.closed {
                current.closed = true;
            }

            if let Some(mut p) = parsing_nodes.pop() {
                if p.is_unclosed_function() {
                    p.closed = true;
                    p.add_child(current);
                    parsing_nodes.push(p);
                } else if p.is_unclosed_arithmetic() {
                    p.add_child(current);
                    parsing_nodes.push(p);
                } else {
                    parsing_nodes.push(p);
                    parsing_nodes.push(current);
                }
            } else {
                parsing_nodes.push(current);
            }
            break;
        } else if !prev.closed {
            prev.add_child(current);
            if prev.is_enough() {
                prev.closed = true;
            }

            if !parsing_nodes.is_empty() {
                parsing_nodes.push(prev);
            } else {
                return Err(Error::StartWithNonValueOperator);
            }
        } else {
            return Err(Error::StartWithNonValueOperator);
        }
    }

    Ok(())
}

fn close_comma(parsing_nodes: &mut Vec<Node>) -> Result<(), Error> {
    if parsing_nodes.len() < 2 {
        return Err(Error::CommaNotWithFunction);
    }

    loop {
        if parsing_nodes.len() < 2 {
            return Err(Error::CommaNotWithFunction);
        }
        let current = parsing_nodes.pop().unwrap();
        let mut prev = parsing_nodes.pop().unwrap();

        if current.operator == Operator::Comma {
            parsing_nodes.push(prev);
            break;
        } else if current.operator.is_left() {
            parsing_nodes.push(prev);
            parsing_nodes.push(current);
            break;
        } else if prev.operator.is_left() {
            if let Some(mut p) = parsing_nodes.pop() {
                if p.is_unclosed_function() {
                    p.add_child(current);
                    parsing_nodes.push(p);
                    parsing_nodes.push(prev);
                    break;
                } else {
                    return Err(Error::CommaNotWithFunction);
                }
            } else {
                return Err(Error::CommaNotWithFunction);
            }
        } else if !prev.closed {
            prev.add_child(current);
            if prev.is_enough() {
                prev.closed = true;
            }

            if !parsing_nodes.is_empty() {
                parsing_nodes.push(prev);
            } else {
                return Err(Error::StartWithNonValueOperator);
            }
        } else {
            return Err(Error::StartWithNonValueOperator);
        }
    }
    Ok(())
}

fn rob_to(mut was_robed: Node, mut robber: Node) -> Vec<Node> {
    let move_out_node = was_robed.move_out_last_node();
    robber.add_child(move_out_node);
    vec![was_robed, robber]
}

fn find(contexts: &[Context], key: &str) -> Option<Value> {
    for context in contexts.iter().rev() {
        match context.get(key) {
            Some(value) => return Some(value.clone()),
            None => continue,
        }
    }

    None
}

fn is_range(ident: &str) -> bool {
    ident.contains("..")
}

const MAX_RANGE_SIZE: i64 = 1_000_000;

fn parse_range(ident: &str) -> Result<Value, Error> {
    match ident.split_once("..") {
        Some((s, e)) if !e.contains("..") => {
            match (s.parse::<i64>(), e.parse::<i64>()) {
                (Ok(start), Ok(end)) => {
                    let size = end.checked_sub(start).unwrap_or(i64::MAX);
                    if size > MAX_RANGE_SIZE {
                        return Err(Error::InvalidRange(ident.to_owned()));
                    }
                    Ok(to_value((start..end).collect::<Vec<i64>>()))
                }
                _ => Err(Error::InvalidRange(ident.to_owned())),
            }
        }
        _ => Err(Error::InvalidRange(ident.to_owned())),
    }
}

fn parse_number(ident: &str) -> Option<Value> {
    let number = ident.parse::<u64>();
    if let Ok(n) = number {
        return Some(to_value(n));
    }

    let number = ident.parse::<i64>();
    if let Ok(n) = number {
        return Some(to_value(n));
    }

    let number = ident.parse::<f64>();
    if let Ok(n) = number {
        return Some(to_value(n));
    }

    None
}
