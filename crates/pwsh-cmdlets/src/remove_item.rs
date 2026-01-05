/// Remove-Item cmdlet - deletes a file or directory
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
            "Remove-Item requires a path".to_string(),
        ))
    }
}

fn parse_recurse(context: &CmdletContext) -> bool {
    get_parameter_ci(context, "Recurse")
        .map(|v| v.to_bool())
        .unwrap_or(false)
}

/// Remove-Item cmdlet deletes a file or directory
pub struct RemoveItemCmdlet;

impl Cmdlet for RemoveItemCmdlet {
    fn name(&self) -> &str {
        "Remove-Item"
    }

    fn execute(
        &self,
        context: CmdletContext,
        _evaluator: &mut pwsh_runtime::Evaluator,
    ) -> Result<Vec<Value>, RuntimeError> {
        let path = extract_path(&context)?;
        let recurse = parse_recurse(&context);

        let metadata = std::fs::metadata(&path).map_err(|e| {
            RuntimeError::InvalidOperation(format!(
                "Failed to access path '{}': {}",
                path.display(),
                e
            ))
        })?;

        if metadata.is_dir() {
            if recurse {
                std::fs::remove_dir_all(&path).map_err(|e| {
                    RuntimeError::InvalidOperation(format!(
                        "Failed to remove directory '{}': {}",
                        path.display(),
                        e
                    ))
                })?;
            } else {
                std::fs::remove_dir(&path).map_err(|e| {
                    RuntimeError::InvalidOperation(format!(
                        "Failed to remove directory '{}': {}",
                        path.display(),
                        e
                    ))
                })?;
            }
        } else {
            std::fs::remove_file(&path).map_err(|e| {
                RuntimeError::InvalidOperation(format!(
                    "Failed to remove file '{}': {}",
                    path.display(),
                    e
                ))
            })?;
        }

        // Remove-Item does not emit pipeline output by default.
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_remove_item_deletes_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("a.txt");
        fs::write(&file_path, "hello").unwrap();

        let cmdlet = RemoveItemCmdlet;
        let context = CmdletContext::new().with_parameter(
            "Path".to_string(),
            Value::String(file_path.to_string_lossy().to_string()),
        );
        let mut evaluator = pwsh_runtime::Evaluator::new();
        cmdlet.execute(context, &mut evaluator).unwrap();

        assert!(!file_path.exists());
    }

    #[test]
    fn test_remove_item_deletes_directory_recurse() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("dir");
        fs::create_dir_all(dir_path.join("sub")).unwrap();
        fs::write(dir_path.join("sub").join("x.txt"), "x").unwrap();

        let cmdlet = RemoveItemCmdlet;
        let context = CmdletContext::new()
            .with_parameter(
                "Path".to_string(),
                Value::String(dir_path.to_string_lossy().to_string()),
            )
            .with_parameter("Recurse".to_string(), Value::Boolean(true));
        let mut evaluator = pwsh_runtime::Evaluator::new();
        cmdlet.execute(context, &mut evaluator).unwrap();

        assert!(!dir_path.exists());
    }
}
