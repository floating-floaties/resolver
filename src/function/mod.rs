
use std::fmt;
use serde_json::Value;

use crate::error::Error;


/// Custom function
pub struct Function {
    /// Maximum number of arguments.
    pub max_args: Option<usize>,
    /// Minimum number of arguments.
    pub min_args: Option<usize>,
    /// Accept values and return a result which contains a value.
    pub compiled: Box<dyn Fn(Vec<Value>) -> Result<Value, Error> + Sync + Send>,
}

impl Function {
    /// Create a function with a closure.
    pub fn new<F>(closure: F) -> Self
        where F: 'static + Fn(Vec<Value>) -> Result<Value, Error> + Sync + Send
    {
        Function {
            max_args: None,
            min_args: None,
            compiled: Box::new(closure),
        }
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Function {{ max_args: {:?}, min_args: {:?} }}",
               self.max_args,
               self.min_args)
    }
}

pub type StaticFunction = fn(Vec<Value>) -> Result<Value, Error>;

/// Custom function
#[derive(Clone, Copy)]
pub struct ConstFunction {
    /// Maximum number of arguments.
    pub max_args: Option<usize>,
    /// Minimum number of arguments.
    pub min_args: Option<usize>,
    /// Accept values and return a result which contains a value.
    pub compiled: StaticFunction,
}

impl ConstFunction {
    /// Create a function with a closure.
    pub fn new(closure: StaticFunction) -> Self
    {
        ConstFunction {
            max_args: None,
            min_args: None,
            compiled: closure,
        }
    }
}

impl fmt::Debug for ConstFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Function {{ max_args: {:?}, min_args: {:?} }}",
               self.max_args,
               self.min_args)
    }
}

