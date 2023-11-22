//! Utilities for managing incoming signals

use std::sync::RwLock;

/// A structure to handle incoming signals and dispatch them to the newest signal handler
#[derive(Default)]
pub struct SignalDispatcher {
    /// All the handlers
    handlers: RwLock<Vec<Box<dyn FnMut() + Send + Sync>>>,
}

impl SignalDispatcher {
    /// Pushes a new handler function to invoke when a signal arrives
    /// # Arguments
    /// * `function` - The function to invoke
    /// # Returns
    /// A handler guard that automatically pops the handler when the guard is dropped
    #[must_use]
    pub fn add_handler(&self, function: Box<dyn FnMut() + Send + Sync>) -> HandlerGuard {
        self.handlers
            .write()
            .expect("Poisoned signal handler collection")
            .push(function);

        HandlerGuard { dispatcher: self }
    }

    /// Pops the last-registered handler
    pub fn pop_last_handler(&self) {
        self.handlers
            .write()
            .expect("Poisoned signal handler collection")
            .pop();
    }

    /// Invoke the handler function of the top-most registered handler
    pub fn handle(&self) {
        if let Some(h) = self
            .handlers
            .write()
            .expect("Poisoned signal handler collection")
            .last_mut()
        {
            h()
        }
    }
}

/// A structure that automatically drops the top-most handler
/// function from the dispatcher when the object is dropped
pub struct HandlerGuard<'a> {
    dispatcher: &'a SignalDispatcher,
}

impl<'a> Drop for HandlerGuard<'a> {
    fn drop(&mut self) {
        self.dispatcher.pop_last_handler()
    }
}
