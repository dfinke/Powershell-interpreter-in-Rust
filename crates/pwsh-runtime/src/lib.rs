/// PowerShell Runtime - Execution environment for PowerShell code
///
/// This module provides the runtime evaluation engine for PowerShell,
/// including value representation, scope management, and expression/statement evaluation.
mod cmdlet;
mod error;
mod evaluator;
mod pipeline;
mod scope;
mod value;

// Public API
pub use cmdlet::{Cmdlet, CmdletContext, CmdletRegistry};
pub use error::RuntimeError;
pub use evaluator::{EvalResult, Evaluator};
pub use pipeline::PipelineExecutor;
pub use scope::{Scope, ScopeStack};
pub use value::Value;
