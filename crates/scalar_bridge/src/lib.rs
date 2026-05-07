pub mod bindings;

use std::rc::Rc;
use std::cell::RefCell;
use scalar_lang::{Environment};
use ferrous_engine::Renderer;

/// Bridge between the Scalar language and the Ferrous Engine.
/// 
/// This struct follows the Strict Modularity principle by delegating
/// binding registration to specialized submodules in [`bindings`].
pub struct Bridge {
    pub renderer: Rc<RefCell<Renderer>>,
}

impl Bridge {
    /// Creates a new Bridge with the given renderer.
    pub fn new(renderer: Renderer) -> Self {
        Self {
            renderer: Rc::new(RefCell::new(renderer)),
        }
    }

    /// Registers all native Scalar functions into the environment.
    /// 
    /// # Example
    /// ```rust,ignore
    /// let bridge = Bridge::new(renderer);
    /// let mut env = Environment::new();
    /// bridge.register_functions(&mut env);
    /// ```
    pub fn register_functions(&self, env: &mut Environment) {
        bindings::animation::register(env, self.renderer.clone());
        bindings::mesh::register(env, self.renderer.clone());
        bindings::shapes::register(env, self.renderer.clone());
    }
}
