
use serde::{
    Serialize,
    Serializer,
    Deserialize,
    Deserializer,
};

use std::cell::RefCell;
use std::rc::Rc;
use std::{fmt, cmp};

use crate::function::{StaticFunction, ConstFunction};
use crate::tree::Tree;
use crate::error::Error;
use crate::{to_value, ConstFunctions};
use crate::{Function, Functions, Context, Contexts, Compiled, Value};

/// Expression builder
pub struct Expr {
    expression: String,
    compiled: Option<Compiled>,
    functions: Functions,
    const_functions: Rc<RefCell<ConstFunctions>>,
    contexts: Contexts,
}

impl Expr {
    /// Create an expression.
    pub fn new<T: Into<String>>(expr: T) -> Expr {
        Expr {
            expression: expr.into(),
            compiled: None,
            functions: Functions::new(),
            const_functions: Rc::from(RefCell::from(ConstFunctions::new())),
            contexts: create_empty_contexts(),
        }
    }

    /// Set function. This functions NOT be cloned. Have highest priority.
    pub fn function<T, F>(mut self, name: T, function: F) -> Expr
        where T: Into<String>,
              F: 'static + Fn(Vec<Value>) -> Result<Value, Error> + Sync + Send
    {
        self.functions.insert(name.into(), Function::new(function));
        self
    }

    /// Set const function. This functions be cloned. Have lowest priority. 
    pub fn const_function<T>(self, name: T, function: StaticFunction)->Expr
    where T: Into<String>{
        self.const_functions.borrow_mut().insert(name.into(), ConstFunction::new(function));
        self
    }

    /// Set value.
    pub fn value<T, V>(mut self, name: T, value: V) -> Expr
        where T: Into<String>,
              V: Serialize
    {
        self.contexts.last_mut().unwrap().insert(name.into(), to_value(value));
        self
    }

    /// Compile an expression.
    /// An expression can be compiled only once and then invoked multiple times with different context and function.
    /// You can also execute a expression without compile.
    pub fn compile(mut self) -> Result<Expr, Error> {
        self.compiled = Some(Tree::new(self.expression.clone()).compile()?);
        Ok(self)
    }

    /// Execute the expression.
    pub fn exec(&mut self) -> Result<Value, Error> {
        if self.compiled.is_none() {
            Tree::new(self.expression.clone()).compile()?(&self.contexts, &self.functions, Rc::clone(&self.const_functions))
        } else {
            self.compiled.as_ref().unwrap()(&self.contexts, &self.functions, Rc::clone(&self.const_functions))
        }
    }

    /// Get reference to compiled object
    pub fn get_compiled(&self) -> Option<&Compiled> {
        self.compiled.as_ref()
    }
}

impl Clone for Expr {
    /// Returns a copy of the value. Notice that functions can not be cloned. The cloned expr's functions will be empty.
    fn clone(&self) -> Expr {
        Expr {
            expression: self.expression.clone(),
            compiled: if self.compiled.is_some() {
                Some(Tree::new(self.expression.clone()).compile().unwrap())
            } else {
                None
            },
            contexts: self.contexts.clone(),
            functions: Functions::new(),
            const_functions: Rc::clone(&self.const_functions)
        }
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "{:?}", self.expression)
    }
}

impl cmp::PartialEq for Expr {
    fn eq(&self, other: &Expr) -> bool {
        self.expression == other.expression
    }
}

impl Serialize for Expr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(format!("{:?}", self).as_str())
    }
}

impl<'de> Deserialize<'de> for Expr {
    fn deserialize<D>(deserializer: D) -> Result<Expr, D::Error>
        where
            D: Deserializer<'de>,
    {
        String::deserialize(deserializer)
            .and_then(|expr| Expr::new(expr).compile().map_err(serde::de::Error::custom))
    }
}


/// Execute options
pub struct ExecOptions<'a> {
    expr: &'a Expr,
    contexts: Option<&'a [Context]>,
    functions: Option<&'a Functions>,
    const_functions:  Rc<RefCell<ConstFunctions>>
}

impl<'a> ExecOptions<'a> {
    /// Create an option.
    pub fn new(expr: &'a Expr) -> ExecOptions<'a> {
        let cf = Rc::clone(&expr.const_functions);
        ExecOptions {
            expr,
            contexts: None,
            functions: None,
            const_functions: cf
        }
    }

    /// Set contexts.
    pub fn contexts(&mut self, contexts: &'a [Context]) -> &'a mut ExecOptions {
        self.contexts = Some(contexts);
        self
    }

    /// Set functions.
    pub fn functions(&mut self, functions: &'a Functions) -> &'a mut ExecOptions {
        self.functions = Some(functions);
        self
    }

    /// Execute the compiled expression.
    pub fn exec(&self) -> Result<Value, Error> {
        let empty_contexts = create_empty_contexts();
        let empty_functions = Functions::new();

        let contexts = if self.contexts.is_some() {
            self.contexts.unwrap()
        } else {
            &empty_contexts
        };

        let functions = if self.functions.is_some() {
            self.functions.unwrap()
        } else {
            &empty_functions
        };

        let compiled = self.expr.get_compiled();
        if let Some (c) = compiled {
            (c)(contexts, functions, Rc::clone(&self.const_functions))
        } else {
            Tree::new(self.expr.expression.clone()).compile()?(contexts, functions,Rc::clone(&self.const_functions))
        }
    }
}


fn create_empty_contexts() -> Contexts {
    let contexts = vec![Context::new()];
    contexts
}