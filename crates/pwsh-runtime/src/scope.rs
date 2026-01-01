/// Scope management for variable storage
use crate::value::Value;
use std::collections::HashMap;

/// A single scope containing variable bindings
#[derive(Debug, Clone)]
pub struct Scope {
    variables: HashMap<String, Value>,
}

impl Scope {
    /// Create a new empty scope
    pub fn new() -> Self {
        Scope {
            variables: HashMap::new(),
        }
    }

    /// Get a variable from this scope (case-insensitive for PowerShell compatibility)
    pub fn get(&self, name: &str) -> Option<&Value> {
        // Try exact match first for performance
        if let Some(value) = self.variables.get(name) {
            return Some(value);
        }
        
        // Fall back to case-insensitive search
        let name_lower = name.to_lowercase();
        self.variables
            .iter()
            .find(|(k, _)| k.to_lowercase() == name_lower)
            .map(|(_, v)| v)
    }

    /// Set a variable in this scope (case-insensitive for PowerShell compatibility)
    pub fn set(&mut self, name: &str, value: Value) {
        // Try exact match first
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            return;
        }
        
        // Check for case-insensitive match
        let name_lower = name.to_lowercase();
        if let Some(existing_key) = self.variables.keys()
            .find(|k| k.to_lowercase() == name_lower)
            .map(|k| k.clone()) {
            // Update with the existing key's case
            self.variables.insert(existing_key, value);
        } else {
            // New variable, use the provided case
            self.variables.insert(name.to_string(), value);
        }
    }

    /// Check if a variable exists in this scope (case-insensitive for PowerShell compatibility)
    pub fn contains(&self, name: &str) -> bool {
        // Try exact match first for performance
        if self.variables.contains_key(name) {
            return true;
        }
        
        // Fall back to case-insensitive search
        let name_lower = name.to_lowercase();
        self.variables.keys().any(|k| k.to_lowercase() == name_lower)
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

/// Stack of scopes for nested contexts (functions, blocks, etc.)
#[derive(Debug)]
pub struct ScopeStack {
    scopes: Vec<Scope>,
}

impl ScopeStack {
    /// Create a new scope stack with a global scope
    pub fn new() -> Self {
        ScopeStack {
            scopes: vec![Scope::new()],
        }
    }

    /// Push a new scope onto the stack
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    /// Pop the current scope from the stack
    /// Returns None if trying to pop the global scope
    pub fn pop_scope(&mut self) -> Option<Scope> {
        if self.scopes.len() > 1 {
            self.scopes.pop()
        } else {
            None // Don't allow popping the global scope
        }
    }

    /// Get a variable, searching from innermost to outermost scope
    pub fn get_variable(&self, name: &str) -> Option<Value> {
        // Search from innermost (last) to outermost (first)
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    /// Set a variable in the appropriate scope
    /// If the variable exists in any scope, update it there
    /// Otherwise, create it in the current (innermost) scope
    pub fn set_variable(&mut self, name: &str, value: Value) {
        // Search for existing variable from innermost to outermost
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains(name) {
                scope.set(name, value);
                return;
            }
        }

        // Variable doesn't exist, create it in current scope
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.set(name, value);
        }
    }

    /// Define a variable in the current (innermost) scope
    /// This always creates or updates in the current scope, even if it exists in outer scopes
    pub fn define_variable(&mut self, name: &str, value: Value) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.set(name, value);
        }
    }

    /// Get the depth of the scope stack
    pub fn depth(&self) -> usize {
        self.scopes.len()
    }
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_basic_operations() {
        let mut scope = Scope::new();

        // Initially empty
        assert!(scope.get("x").is_none());

        // Set and get
        scope.set("x", Value::Number(42.0));
        assert_eq!(scope.get("x"), Some(&Value::Number(42.0)));

        // Update
        scope.set("x", Value::Number(100.0));
        assert_eq!(scope.get("x"), Some(&Value::Number(100.0)));
    }

    #[test]
    fn test_scope_stack_single_scope() {
        let mut stack = ScopeStack::new();

        // Set variable
        stack.set_variable("x", Value::Number(5.0));
        assert_eq!(stack.get_variable("x"), Some(Value::Number(5.0)));

        // Update variable
        stack.set_variable("x", Value::Number(10.0));
        assert_eq!(stack.get_variable("x"), Some(Value::Number(10.0)));
    }

    #[test]
    fn test_scope_stack_nested_scopes() {
        let mut stack = ScopeStack::new();

        // Set in global scope
        stack.set_variable("x", Value::Number(1.0));

        // Push new scope
        stack.push_scope();

        // Can still see outer variable
        assert_eq!(stack.get_variable("x"), Some(Value::Number(1.0)));

        // Set new variable in inner scope
        stack.set_variable("y", Value::Number(2.0));
        assert_eq!(stack.get_variable("y"), Some(Value::Number(2.0)));

        // Update outer variable from inner scope
        stack.set_variable("x", Value::Number(10.0));
        assert_eq!(stack.get_variable("x"), Some(Value::Number(10.0)));

        // Pop scope
        stack.pop_scope();

        // Outer variable still updated
        assert_eq!(stack.get_variable("x"), Some(Value::Number(10.0)));

        // Inner variable no longer accessible
        assert_eq!(stack.get_variable("y"), None);
    }

    #[test]
    fn test_scope_stack_shadowing() {
        let mut stack = ScopeStack::new();

        // Set in global scope
        stack.set_variable("x", Value::Number(1.0));

        // Push new scope
        stack.push_scope();

        // Define same variable in inner scope (shadowing)
        stack.define_variable("x", Value::Number(2.0));
        assert_eq!(stack.get_variable("x"), Some(Value::Number(2.0)));

        // Pop scope
        stack.pop_scope();

        // Back to outer variable
        assert_eq!(stack.get_variable("x"), Some(Value::Number(1.0)));
    }

    #[test]
    fn test_scope_stack_cannot_pop_global() {
        let mut stack = ScopeStack::new();
        assert_eq!(stack.depth(), 1);

        // Cannot pop global scope
        assert!(stack.pop_scope().is_none());
        assert_eq!(stack.depth(), 1);

        // Can pop non-global scopes
        stack.push_scope();
        assert_eq!(stack.depth(), 2);
        assert!(stack.pop_scope().is_some());
        assert_eq!(stack.depth(), 1);
    }
}
