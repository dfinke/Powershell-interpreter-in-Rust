/// Test-Path cmdlet - checks if a file system path exists
use pwsh_runtime::{Cmdlet, CmdletContext, RuntimeError, Value};
use std::path::PathBuf;

fn get_parameter_ci<'a>(context: &'a CmdletContext, name: &str) -> Option<&'a Value> {
    // Try exact match first
    if let Some(v) = context.parameters.get(name) {
        return Some(v);
    }

    let name_lower = name.to_lowercase();
    context
        .parameters
        .iter()
        .find(|(k, _)| k.to_lowercase() == name_lower)
        .map(|(_, v)| v)
}

fn resolve_path(path: &str) -> Result<PathBuf, RuntimeError> {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        return Ok(p);
    }

    let cwd = std::env::current_dir().map_err(|e| {
        RuntimeError::InvalidOperation(format!("Failed to get current directory: {}", e))
    })?;
    Ok(cwd.join(p))
}

fn extract_path(context: &CmdletContext) -> Result<PathBuf, RuntimeError> {
    if let Some(Value::String(p)) = get_parameter_ci(context, "Path") {
        resolve_path(p)
    } else if let Some(Value::String(p)) = context.get_argument(0) {
        resolve_path(p)
    } else {
        Err(RuntimeError::InvalidOperation(
            "Test-Path requires a path".to_string(),
        ))
    }
}

/// Test-Path cmdlet checks for file/directory existence
pub struct TestPathCmdlet;

impl Cmdlet for TestPathCmdlet {
    fn name(&self) -> &str {
        "Test-Path"
    }

    fn execute(
        &self,
        context: CmdletContext,
        _evaluator: &mut pwsh_runtime::Evaluator,
    ) -> Result<Vec<Value>, RuntimeError> {
        let path = extract_path(&context)?;

        let exists = match std::fs::metadata(&path) {
            Ok(_) => true,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
            Err(e) => {
                return Err(RuntimeError::InvalidOperation(format!(
                    "Failed to check path '{}': {}",
                    path.display(),
                    e
                )))
            }
        };

        Ok(vec![Value::Boolean(exists)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_test_path_true_for_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("a.txt");
        fs::write(&file_path, "hello").unwrap();

        let cmdlet = TestPathCmdlet;
        let context = CmdletContext::new().with_parameter(
            "Path".to_string(),
            Value::String(file_path.to_string_lossy().to_string()),
        );
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result, vec![Value::Boolean(true)]);
    }

    #[test]
    fn test_test_path_false_for_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("missing.txt");

        let cmdlet = TestPathCmdlet;
        let context = CmdletContext::new().with_arguments(vec![Value::String(
            file_path.to_string_lossy().to_string(),
        )]);
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result, vec![Value::Boolean(false)]);
    }
}
