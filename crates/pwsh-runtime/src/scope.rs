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

    /// Find a key in the variables map with case-insensitive comparison
    /// Returns the actual key if found, None otherwise
    fn find_key_case_insensitive(&self, name: &str) -> Option<&String> {
        let name_lower = name.to_lowercase();
        self.variables
            .keys()
            .find(|k| k.to_lowercase() == name_lower)
    }

    /// Get a variable from this scope (case-insensitive for PowerShell compatibility)
    pub fn get(&self, name: &str) -> Option<&Value> {
        // Try exact match first for performance
        if let Some(value) = self.variables.get(name) {
            return Some(value);
        }

        // Fall back to case-insensitive search
        self.find_key_case_insensitive(name)
            .and_then(|key| self.variables.get(key))
    }

    /// Set a variable in this scope (case-insensitive for PowerShell compatibility)
    pub fn set(&mut self, name: &str, value: Value) {
        // Try exact match first
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            return;
        }

        // Check for case-insensitive match
        if let Some(existing_key) = self.find_key_case_insensitive(name).cloned() {
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
        self.find_key_case_insensitive(name).is_some()
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

    /// Parse a variable name into scope qualifier and base name
    /// Returns (scope_qualifier, base_name) where scope_qualifier is Some("global"|"local"|"script") or None
    fn parse_scope_qualifier(name: &str) -> (Option<&str>, &str) {
        if let Some(colon_pos) = name.find(':') {
            let (qualifier, rest) = name.split_at(colon_pos);
            let base_name = &rest[1..]; // Skip the ':'
            
            // Only recognize valid scope qualifiers
            match qualifier.to_lowercase().as_str() {
                "global" | "local" | "script" => (Some(qualifier), base_name),
                _ => (None, name), // Invalid qualifier, treat as regular variable name
            }
        } else {
            (None, name)
        }
    }

    /// Get a variable with scope qualifier support
    /// Supports $global:x, $local:y, $script:z
    pub fn get_variable_qualified(&self, name: &str) -> Option<Value> {
        let (qualifier, base_name) = Self::parse_scope_qualifier(name);
        
        match qualifier {
            Some("global") => {
                // Get from global scope (first scope)
                self.scopes.first()?.get(base_name).cloned()
            }
            Some("local") => {
                // Get from current/local scope (last scope)
                self.scopes.last()?.get(base_name).cloned()
            }
            Some("script") => {
                // For now, treat script scope as global scope
                // In a full implementation, script scope would be the top-level scope of the current script file
                self.scopes.first()?.get(base_name).cloned()
            }
            _ => {
                // No qualifier or invalid qualifier - use normal lookup
                self.get_variable(base_name)
            }
        }
    }

    /// Set a variable with scope qualifier support
    /// Supports $global:x = value, $local:y = value, $script:z = value
    pub fn set_variable_qualified(&mut self, name: &str, value: Value) {
        let (qualifier, base_name) = Self::parse_scope_qualifier(name);
        
        match qualifier {
            Some("global") => {
                // Set in global scope (first scope)
                if let Some(global_scope) = self.scopes.first_mut() {
                    global_scope.set(base_name, value);
                }
            }
            Some("local") => {
                // Set in current/local scope (last scope)
                if let Some(local_scope) = self.scopes.last_mut() {
                    local_scope.set(base_name, value);
                }
            }
            Some("script") => {
                // For now, treat script scope as global scope
                if let Some(global_scope) = self.scopes.first_mut() {
                    global_scope.set(base_name, value);
                }
            }
            _ => {
                // No qualifier or invalid qualifier - use normal set
                self.set_variable(base_name, value);
            }
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

    // Week 8: Scope qualifier tests
    #[test]
    fn test_global_scope_qualifier() {
        let mut stack = ScopeStack::new();
        
        // Set in global scope
        stack.set_variable_qualified("global:x", Value::Number(5.0));
        assert_eq!(stack.get_variable_qualified("global:x"), Some(Value::Number(5.0)));
        
        // Push new scope
        stack.push_scope();
        
        // Global variable accessible from inner scope
        assert_eq!(stack.get_variable_qualified("global:x"), Some(Value::Number(5.0)));
        
        // Update global variable from inner scope
        stack.set_variable_qualified("global:x", Value::Number(10.0));
        
        // Pop scope
        stack.pop_scope();
        
        // Global variable was updated
        assert_eq!(stack.get_variable_qualified("global:x"), Some(Value::Number(10.0)));
    }

    #[test]
    fn test_local_scope_qualifier() {
        let mut stack = ScopeStack::new();
        
        // Set in global scope
        stack.set_variable("x", Value::Number(1.0));
        
        // Push new scope
        stack.push_scope();
        
        // Set local variable (in current scope)
        stack.set_variable_qualified("local:x", Value::Number(2.0));
        
        // Local access returns the local value
        assert_eq!(stack.get_variable_qualified("local:x"), Some(Value::Number(2.0)));
        
        // Global access returns the global value
        assert_eq!(stack.get_variable_qualified("global:x"), Some(Value::Number(1.0)));
        
        // Regular access returns the local (shadowing) value
        assert_eq!(stack.get_variable("x"), Some(Value::Number(2.0)));
        
        // Pop scope
        stack.pop_scope();
        
        // Back to global value
        assert_eq!(stack.get_variable("x"), Some(Value::Number(1.0)));
    }

    #[test]
    fn test_script_scope_qualifier() {
        let mut stack = ScopeStack::new();
        
        // Script scope currently behaves like global scope
        stack.set_variable_qualified("script:z", Value::Number(100.0));
        assert_eq!(stack.get_variable_qualified("script:z"), Some(Value::Number(100.0)));
        assert_eq!(stack.get_variable_qualified("global:z"), Some(Value::Number(100.0)));
    }

    #[test]
    fn test_scope_qualifier_parsing() {
        let mut stack = ScopeStack::new();
        
        // Test that scope qualifiers are case-insensitive
        stack.set_variable_qualified("GLOBAL:upper", Value::Number(1.0));
        assert_eq!(stack.get_variable_qualified("global:upper"), Some(Value::Number(1.0)));
        
        stack.set_variable_qualified("Global:mixed", Value::Number(2.0));
        assert_eq!(stack.get_variable_qualified("global:mixed"), Some(Value::Number(2.0)));
    }

    #[test]
    fn test_invalid_scope_qualifier() {
        let mut stack = ScopeStack::new();
        
        // Invalid qualifier should be treated as part of the variable name
        stack.set_variable_qualified("invalid:name", Value::Number(42.0));
        
        // Should be stored as "invalid:name" (whole string)
        assert_eq!(stack.get_variable("invalid:name"), Some(Value::Number(42.0)));
    }

    #[test]
    fn test_scope_qualifier_with_nested_scopes() {
        let mut stack = ScopeStack::new();
        
        // Set global variable
        stack.set_variable_qualified("global:counter", Value::Number(0.0));
        
        // Push scope and increment
        stack.push_scope();
        stack.set_variable_qualified("global:counter", Value::Number(1.0));
        
        // Push another scope and increment
        stack.push_scope();
        stack.set_variable_qualified("global:counter", Value::Number(2.0));
        
        // Pop both scopes
        stack.pop_scope();
        stack.pop_scope();
        
        // Global variable has final value
        assert_eq!(stack.get_variable_qualified("global:counter"), Some(Value::Number(2.0)));
    }
}
