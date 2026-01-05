/// New-Item cmdlet - creates a file or directory
use pwsh_runtime::{Cmdlet, CmdletContext, RuntimeError, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
            "New-Item requires a path".to_string(),
        ))
    }
}

fn parse_item_type(context: &CmdletContext) -> Result<String, RuntimeError> {
    // Support both -Type (from our plan) and the standard PowerShell -ItemType.
    let v = get_parameter_ci(context, "Type")
        .or_else(|| get_parameter_ci(context, "ItemType"))
        .or_else(|| context.get_argument(1));

    let Some(v) = v else {
        return Err(RuntimeError::InvalidOperation(
            "New-Item requires -Type (File or Directory)".to_string(),
        ));
    };

    let s = match v {
        Value::String(s) => s.trim().to_string(),
        other => other.to_string(),
    };

    Ok(s)
}

fn parse_force(context: &CmdletContext) -> bool {
    get_parameter_ci(context, "Force")
        .map(|v| v.to_bool())
        .unwrap_or(false)
}

fn build_item_object(path: &Path, item_type: &str, is_dir: bool) -> Value {
    let mut props = HashMap::new();
    props.insert(
        "FullName".to_string(),
        Value::String(path.to_string_lossy().to_string()),
    );
    props.insert(
        "Name".to_string(),
        Value::String(
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string()),
        ),
    );
    props.insert("ItemType".to_string(), Value::String(item_type.to_string()));
    props.insert("Directory".to_string(), Value::Boolean(is_dir));
    Value::Object(props)
}

/// New-Item cmdlet creates a file or directory
pub struct NewItemCmdlet;

impl Cmdlet for NewItemCmdlet {
    fn name(&self) -> &str {
        "New-Item"
    }

    fn execute(
        &self,
        context: CmdletContext,
        _evaluator: &mut pwsh_runtime::Evaluator,
    ) -> Result<Vec<Value>, RuntimeError> {
        let path = extract_path(&context)?;
        let item_type = parse_item_type(&context)?;
        let force = parse_force(&context);

        let item_type_norm = item_type.trim().to_ascii_lowercase();
        match item_type_norm.as_str() {
            "directory" | "dir" | "folder" => {
                if path.exists() {
                    if !force {
                        return Err(RuntimeError::InvalidOperation(format!(
                            "Path already exists: {}",
                            path.display()
                        )));
                    }
                } else {
                    fs::create_dir_all(&path).map_err(|e| {
                        RuntimeError::InvalidOperation(format!(
                            "Failed to create directory '{}': {}",
                            path.display(),
                            e
                        ))
                    })?;
                }

                Ok(vec![build_item_object(&path, "Directory", true)])
            }
            "file" => {
                if let Some(parent) = path.parent() {
                    if !parent.as_os_str().is_empty() && !parent.exists() {
                        return Err(RuntimeError::InvalidOperation(format!(
                            "Parent directory does not exist: {}",
                            parent.display()
                        )));
                    }
                }

                if path.exists() && !force {
                    return Err(RuntimeError::InvalidOperation(format!(
                        "Path already exists: {}",
                        path.display()
                    )));
                }

                fs::File::create(&path).map_err(|e| {
                    RuntimeError::InvalidOperation(format!(
                        "Failed to create file '{}': {}",
                        path.display(),
                        e
                    ))
                })?;

                Ok(vec![build_item_object(&path, "File", false)])
            }
            other => Err(RuntimeError::InvalidOperation(format!(
                "Unsupported -Type for New-Item: {}",
                other
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_item_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("sub");

        let cmdlet = NewItemCmdlet;
        let context = CmdletContext::new()
            .with_parameter(
                "Path".to_string(),
                Value::String(dir_path.to_string_lossy().to_string()),
            )
            .with_parameter("Type".to_string(), Value::String("Directory".to_string()));
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let _ = cmdlet.execute(context, &mut evaluator).unwrap();

        assert!(dir_path.exists());
        assert!(dir_path.is_dir());
    }

    #[test]
    fn test_new_item_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("a.txt");

        let cmdlet = NewItemCmdlet;
        let context = CmdletContext::new().with_arguments(vec![
            Value::String(file_path.to_string_lossy().to_string()),
            Value::String("File".to_string()),
        ]);
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let _ = cmdlet.execute(context, &mut evaluator).unwrap();

        assert!(file_path.exists());
        assert!(file_path.is_file());
    }
}
