/// PowerShell Runtime - Execution environment for PowerShell code
///
/// This module provides the runtime evaluation engine for PowerShell,
/// including value representation, scope management, and expression/statement evaluation.
mod error;
mod evaluator;
mod scope;
mod value;

// Public API
pub use error::RuntimeError;
pub use evaluator::{EvalResult, Evaluator};
pub use scope::{Scope, ScopeStack};
pub use value::Value;
