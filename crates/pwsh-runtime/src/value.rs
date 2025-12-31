/// PowerShell Value types
use std::collections::HashMap;
use std::fmt;

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
}

impl Value {
    /// Convert value to string representation for display
    fn display_string(&self) -> String {
        match self {
            Value::Null => String::new(),
            Value::Boolean(b) => b.to_string(),
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

    /// Get a property from an object
    pub fn get_property(&self, name: &str) -> Option<Value> {
        match self {
            Value::Object(props) => props.get(name).cloned(),
            _ => None,
        }
    }

    /// Set a property on an object
    pub fn set_property(&mut self, name: &str, value: Value) -> Result<(), String> {
        match self {
            Value::Object(props) => {
                props.insert(name.to_string(), value);
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
        assert_eq!(Value::Boolean(true).to_string(), "true");
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
}
