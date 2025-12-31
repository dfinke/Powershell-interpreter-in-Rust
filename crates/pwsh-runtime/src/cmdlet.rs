/// Cmdlet trait and execution infrastructure
use crate::value::Value;
use crate::error::RuntimeError;
use std::collections::HashMap;

/// Context provided to cmdlets during execution
pub struct CmdletContext {
    /// Input from pipeline (if any)
    pub pipeline_input: Vec<Value>,
    /// Named parameters passed to the cmdlet
    pub parameters: HashMap<String, Value>,
    /// Positional arguments passed to the cmdlet
    pub arguments: Vec<Value>,
}

impl CmdletContext {
    /// Create a new cmdlet context
    pub fn new() -> Self {
        Self {
            pipeline_input: Vec::new(),
            parameters: HashMap::new(),
            arguments: Vec::new(),
        }
    }

    /// Create context with pipeline input
    pub fn with_input(input: Vec<Value>) -> Self {
        Self {
            pipeline_input: input,
            parameters: HashMap::new(),
            arguments: Vec::new(),
        }
    }

    /// Add a named parameter
    pub fn with_parameter(mut self, name: String, value: Value) -> Self {
        self.parameters.insert(name, value);
        self
    }

    /// Add positional arguments
    pub fn with_arguments(mut self, args: Vec<Value>) -> Self {
        self.arguments = args;
        self
    }

    /// Get a named parameter
    pub fn get_parameter(&self, name: &str) -> Option<&Value> {
        self.parameters.get(name)
    }

    /// Get a positional argument by index
    pub fn get_argument(&self, index: usize) -> Option<&Value> {
        self.arguments.get(index)
    }
}

/// Trait that all cmdlets must implement
pub trait Cmdlet: Send + Sync {
    /// Get the name of the cmdlet (e.g., "Write-Output")
    fn name(&self) -> &str;

    /// Execute the cmdlet with the given context
    /// Returns a Vec of output values (for pipeline)
    fn execute(&self, context: CmdletContext) -> Result<Vec<Value>, RuntimeError>;
}

/// Registry for managing cmdlets
pub struct CmdletRegistry {
    cmdlets: HashMap<String, Box<dyn Cmdlet>>,
}

impl CmdletRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            cmdlets: HashMap::new(),
        }
    }

    /// Register a cmdlet
    pub fn register(&mut self, cmdlet: Box<dyn Cmdlet>) {
        let name = cmdlet.name().to_lowercase();
        self.cmdlets.insert(name, cmdlet);
    }

    /// Get a cmdlet by name (case-insensitive)
    pub fn get(&self, name: &str) -> Option<&Box<dyn Cmdlet>> {
        self.cmdlets.get(&name.to_lowercase())
    }

    /// Check if a cmdlet is registered
    pub fn contains(&self, name: &str) -> bool {
        self.cmdlets.contains_key(&name.to_lowercase())
    }
}

impl Default for CmdletRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test cmdlet implementation
    struct TestCmdlet;

    impl Cmdlet for TestCmdlet {
        fn name(&self) -> &str {
            "Test-Cmdlet"
        }

        fn execute(&self, context: CmdletContext) -> Result<Vec<Value>, RuntimeError> {
            Ok(context.pipeline_input)
        }
    }

    #[test]
    fn test_cmdlet_context_creation() {
        let ctx = CmdletContext::new();
        assert_eq!(ctx.pipeline_input.len(), 0);
        assert_eq!(ctx.parameters.len(), 0);
        assert_eq!(ctx.arguments.len(), 0);
    }

    #[test]
    fn test_cmdlet_context_with_input() {
        let input = vec![Value::Number(42.0)];
        let ctx = CmdletContext::with_input(input.clone());
        assert_eq!(ctx.pipeline_input, input);
    }

    #[test]
    fn test_cmdlet_registry() {
        let mut registry = CmdletRegistry::new();
        registry.register(Box::new(TestCmdlet));
        
        assert!(registry.contains("Test-Cmdlet"));
        assert!(registry.contains("test-cmdlet")); // Case-insensitive
        assert!(!registry.contains("NonExistent"));
        
        let cmdlet = registry.get("test-cmdlet");
        assert!(cmdlet.is_some());
    }

    #[test]
    fn test_cmdlet_execution() {
        let cmdlet = TestCmdlet;
        let input = vec![Value::Number(42.0)];
        let ctx = CmdletContext::with_input(input.clone());
        
        let result = cmdlet.execute(ctx).unwrap();
        assert_eq!(result, input);
    }
}
