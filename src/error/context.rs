use std::collections::VecDeque;

/// Context that can be added to a [ALError](super::ALError)
#[derive(Debug, Default)]
pub struct ALErrorContext {
    context: VecDeque<String>,
}

impl ALErrorContext {
    /// Add new context to the error
    /// # Arguments
    /// * `f` - A function that returns a [ToString] containing the error context string
    pub fn add<S: ToString, F: Fn() -> S>(&mut self, f: F) {
        self.context.push_back((f)().to_string())
    }
}
