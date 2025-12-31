/// Runtime error types
use std::fmt;

/// Errors that can occur during runtime evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    /// Variable is not defined
    UndefinedVariable(String),
    /// Type mismatch in operation
    TypeMismatch {
        expected: String,
        got: String,
        operation: String,
    },
    /// Division by zero
    DivisionByZero,
    /// Invalid operation
    InvalidOperation(String),
    /// Return statement outside of function
    ReturnOutsideFunction,
    /// Cannot access property on non-object
    InvalidPropertyAccess(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::UndefinedVariable(name) => {
                write!(f, "Variable '${name}' is not defined")
            }
            RuntimeError::TypeMismatch {
                expected,
                got,
                operation,
            } => {
                write!(
                    f,
                    "Type mismatch in {operation}: expected {expected}, got {got}"
                )
            }
            RuntimeError::DivisionByZero => {
                write!(f, "Division by zero")
            }
            RuntimeError::InvalidOperation(msg) => {
                write!(f, "Invalid operation: {msg}")
            }
            RuntimeError::ReturnOutsideFunction => {
                write!(f, "Return statement outside of function")
            }
            RuntimeError::InvalidPropertyAccess(msg) => {
                write!(f, "Invalid property access: {msg}")
            }
        }
    }
}

impl std::error::Error for RuntimeError {}
