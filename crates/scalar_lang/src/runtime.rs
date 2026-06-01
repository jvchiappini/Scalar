use std::rc::Rc;
use std::collections::HashMap;
use std::fmt;

use crate::ast::Stmt;

/// Represents a value in the Scalar runtime.
#[derive(Clone)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    List(Vec<Value>),
    NodeId(u32),
    NativeFunction(Rc<dyn Fn(Vec<Value>, HashMap<String, Value>) -> Result<Value, String>>),
    String(String),
    Object(HashMap<String, Value>),
    /// A user-defined function (`fn name(params) { body }`).
    Fn {
        params: Vec<String>,
        body: Vec<Stmt>,
        source: String,
    },
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "Number({})", n),
            Value::Boolean(b) => write!(f, "Boolean({})", b),
            Value::List(l) => write!(f, "List({:?})", l),
            Value::NodeId(id) => write!(f, "NodeId({})", id),
            Value::NativeFunction(_) => write!(f, "NativeFunction"),
            Value::String(s) => write!(f, "String({})", s),
            Value::Object(o) => write!(f, "Object({:?})", o.keys()),
            Value::Fn { params, .. } => write!(f, "Fn({})", params.join(", ")),
        }
    }
}

/// Execution environment holding variables and function state.
#[derive(Clone)]
pub struct Environment {
    variables: HashMap<String, Value>,
    parent: Option<Rc<Environment>>,
}

impl Environment {
    /// Creates a new empty environment.
    pub fn new() -> Self {
        let mut variables = HashMap::new();
        
        // v14: Predefined Standard Colors (RGBA)
        variables.insert("WHITE".to_string(), Value::List(vec![Value::Number(1.0), Value::Number(1.0), Value::Number(1.0), Value::Number(1.0)]));
        variables.insert("BLACK".to_string(), Value::List(vec![Value::Number(0.0), Value::Number(0.0), Value::Number(0.0), Value::Number(1.0)]));
        variables.insert("RED".to_string(), Value::List(vec![Value::Number(1.0), Value::Number(0.0), Value::Number(0.0), Value::Number(1.0)]));
        variables.insert("GREEN".to_string(), Value::List(vec![Value::Number(0.0), Value::Number(1.0), Value::Number(0.0), Value::Number(1.0)]));
        variables.insert("BLUE".to_string(), Value::List(vec![Value::Number(0.0), Value::Number(0.0), Value::Number(1.0), Value::Number(1.0)]));
        variables.insert("YELLOW".to_string(), Value::List(vec![Value::Number(1.0), Value::Number(1.0), Value::Number(0.0), Value::Number(1.0)]));
        variables.insert("CYAN".to_string(), Value::List(vec![Value::Number(0.0), Value::Number(1.0), Value::Number(1.0), Value::Number(1.0)]));
        variables.insert("MAGENTA".to_string(), Value::List(vec![Value::Number(1.0), Value::Number(0.0), Value::Number(1.0), Value::Number(1.0)]));
        variables.insert("NONE".to_string(), Value::List(vec![])); // no-fill sentinel
        variables.insert("true".to_string(), Value::Boolean(true));
        variables.insert("false".to_string(), Value::Boolean(false));

        Self {
            variables,
            parent: None,
        }
    }

    /// Sets a variable in the current scope.
    pub fn define(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// Retrieves a variable by searching up the scope chain.
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(val) = self.variables.get(name) {
            return Some(val.clone());
        }
        if let Some(parent) = &self.parent {
            return parent.get(name);
        }
        None
    }

    /// Creates a child scope.
    pub fn child(parent: Rc<Environment>) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(parent),
        }
    }
}
