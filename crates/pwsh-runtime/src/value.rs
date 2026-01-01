/// PowerShell Value types
use std::collections::HashMap;
use std::fmt;

/// Function definition stored as a value
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<pwsh_parser::Parameter>,
    pub body: pwsh_parser::Block,
}

/// Script block stored as a value (anonymous code block)
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptBlock {
    pub body: pwsh_parser::Block,
}

/// A value in the PowerShell runtime
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Null value
    Null,
    /// Boolean value
    Boolean(bool),
    /// Numeric value (PowerShell uses double precision floats)
    Number(f64),
    /// String value
    String(String),
    /// Object with properties
    Object(HashMap<String, Value>),
    /// Array of values
    Array(Vec<Value>),
    /// Function definition
    Function(Function),
    /// Script block (anonymous code block)
    ScriptBlock(ScriptBlock),
}

impl Value {
    /// Convert value to string representation for display
    fn display_string(&self) -> String {
        match self {
            Value::Null => String::new(),
            Value::Boolean(b) => {
                if *b {
                    "True".to_string()
                } else {
                    "False".to_string()
                }
            }
            Value::Number(n) => {
                // Format numbers nicely (avoid unnecessary decimals for whole numbers)
                if n.fract() == 0.0 {
                    format!("{:.0}", n)
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => s.clone(),
            Value::Object(props) => {
                // Simple object representation
                let mut parts: Vec<String> = props
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v.display_string()))
                    .collect();
                parts.sort();
                format!("@{{{}}}", parts.join("; "))
            }
            Value::Array(items) => {
                let parts: Vec<String> = items.iter().map(|v| v.display_string()).collect();
                format!("@({})", parts.join(", "))
            }
            Value::Function(func) => {
                format!("function {}", func.name)
            }
            Value::ScriptBlock(_) => "{ script block }".to_string(),
        }
    }

    /// Convert value to boolean (PowerShell truthiness rules)
    pub fn to_bool(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Object(_) => true,
            Value::Array(items) => !items.is_empty(),
            Value::Function(_) => true,
            Value::ScriptBlock(_) => true,
        }
    }

    /// Try to convert value to number
    pub fn to_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::String(s) => s.parse::<f64>().ok(),
            Value::Boolean(true) => Some(1.0),
            Value::Boolean(false) => Some(0.0),
            _ => None,
        }
    }

    /// Get a property from an object (case-insensitive)
    pub fn get_property(&self, name: &str) -> Option<Value> {
        match self {
            Value::Object(props) => {
                // Try exact match first for performance
                if let Some(value) = props.get(name) {
                    return Some(value.clone());
                }

                // Fall back to case-insensitive property lookup
                let name_lower = name.to_lowercase();
                props
                    .iter()
                    .find(|(k, _)| k.to_lowercase() == name_lower)
                    .map(|(_, v)| v.clone())
            }
            _ => None,
        }
    }

    /// Set a property on an object (case-insensitive - updates existing key or adds new)
    pub fn set_property(&mut self, name: &str, value: Value) -> Result<(), String> {
        match self {
            Value::Object(props) => {
                // Try exact match first for performance
                if props.contains_key(name) {
                    props.insert(name.to_string(), value);
                    return Ok(());
                }

                // Fall back to case-insensitive property update: find existing key or add new one
                let name_lower = name.to_lowercase();
                if let Some(existing_key) = props
                    .keys()
                    .find(|k| k.to_lowercase() == name_lower)
                    .cloned()
                {
                    // Update existing property with original key name
                    props.insert(existing_key, value);
                } else {
                    // Add new property
                    props.insert(name.to_string(), value);
                }
                Ok(())
            }
            _ => Err("Cannot set property on non-object value".to_string()),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_to_string() {
        assert_eq!(Value::Null.to_string(), "");
        assert_eq!(Value::Boolean(true).to_string(), "True");
        assert_eq!(Value::Number(42.0).to_string(), "42");
        assert_eq!(Value::Number(3.15).to_string(), "3.15");
        assert_eq!(Value::String("hello".to_string()).to_string(), "hello");
    }

    #[test]
    fn test_value_to_bool() {
        assert!(!Value::Null.to_bool());
        assert!(Value::Boolean(true).to_bool());
        assert!(!Value::Boolean(false).to_bool());
        assert!(!Value::Number(0.0).to_bool());
        assert!(Value::Number(42.0).to_bool());
        assert!(!Value::String("".to_string()).to_bool());
        assert!(Value::String("hello".to_string()).to_bool());
    }

    #[test]
    fn test_value_to_number() {
        assert_eq!(Value::Number(42.0).to_number(), Some(42.0));
        assert_eq!(Value::String("3.15".to_string()).to_number(), Some(3.15));
        assert_eq!(Value::Boolean(true).to_number(), Some(1.0));
        assert_eq!(Value::Boolean(false).to_number(), Some(0.0));
        assert_eq!(Value::Null.to_number(), None);
    }

    #[test]
    fn test_object_properties() {
        let mut obj = Value::Object(HashMap::new());
        assert!(obj
            .set_property("name", Value::String("test".to_string()))
            .is_ok());
        assert_eq!(
            obj.get_property("name"),
            Some(Value::String("test".to_string()))
        );
        assert_eq!(obj.get_property("missing"), None);
    }

    #[test]
    fn test_object_properties_case_insensitive() {
        let mut obj = Value::Object(HashMap::new());
        // Set property with mixed case
        assert!(obj
            .set_property("Name", Value::String("test".to_string()))
            .is_ok());
        
        // Should retrieve with different case variations
        assert_eq!(
            obj.get_property("name"),
            Some(Value::String("test".to_string()))
        );
        assert_eq!(
            obj.get_property("NAME"),
            Some(Value::String("test".to_string()))
        );
        assert_eq!(
            obj.get_property("Name"),
            Some(Value::String("test".to_string()))
        );
    }

    #[test]
    fn test_object_properties_update_preserves_case() {
        let mut obj = Value::Object(HashMap::new());
        // Set property with specific case
        obj.set_property("Age", Value::Number(30.0)).unwrap();
        
        // Update with different case should preserve original key
        obj.set_property("age", Value::Number(31.0)).unwrap();
        
        // Should still have only one key (the original)
        if let Value::Object(map) = &obj {
            assert_eq!(map.len(), 1);
            assert!(map.contains_key("Age"));
        } else {
            panic!("Expected Object");
        }
        
        // Value should be updated
        assert_eq!(obj.get_property("age"), Some(Value::Number(31.0)));
    }

}
