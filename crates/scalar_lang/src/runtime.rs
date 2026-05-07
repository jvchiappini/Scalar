use std::rc::Rc;
use std::collections::HashMap;
use std::fmt;

/// Represents a value in the Scalar runtime.
#[derive(Clone)]
pub enum Value {
    Number(f64),
    List(Vec<Value>),
    NodeId(u32),
    NativeFunction(Rc<dyn Fn(Vec<Value>) -> Result<Value, String>>),
    String(String),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "Number({})", n),
            Value::List(l) => write!(f, "List({:?})", l),
            Value::NodeId(id) => write!(f, "NodeId({})", id),
            Value::NativeFunction(_) => write!(f, "NativeFunction"),
            Value::String(s) => write!(f, "String({})", s),
        }
    }
}

/// Execution environment holding variables and function state.
pub struct Environment {
    variables: HashMap<String, Value>,
    parent: Option<Rc<Environment>>,
}

impl Environment {
    /// Creates a new empty environment.
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
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
