/// Set-Content cmdlet - writes content to a file (overwriting existing content)
use pwsh_runtime::{Cmdlet, CmdletContext, RuntimeError, Value};
use std::fs;
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

fn normalize_value_to_lines(value: Value) -> Vec<String> {
    match value {
        Value::Array(items) => items.into_iter().map(|v| v.to_string()).collect(),
        other => vec![other.to_string()],
    }
}

fn values_to_file_string(values: Vec<Value>) -> String {
    if values.is_empty() {
        return String::new();
    }

    let mut lines: Vec<String> = Vec::new();
    for v in values {
        lines.extend(normalize_value_to_lines(v));
    }

    if lines.is_empty() {
        String::new()
    } else {
        let mut s = lines.join("\n");
        s.push('\n');
        s
    }
}

/// Set-Content cmdlet writes file contents
pub struct SetContentCmdlet;

impl Cmdlet for SetContentCmdlet {
    fn name(&self) -> &str {
        "Set-Content"
    }

    fn execute(
        &self,
        context: CmdletContext,
        _evaluator: &mut pwsh_runtime::Evaluator,
    ) -> Result<Vec<Value>, RuntimeError> {
        // Path: -Path or first positional argument
        let path = if let Some(Value::String(p)) = get_parameter_ci(&context, "Path") {
            resolve_path(p)?
        } else if let Some(Value::String(p)) = context.get_argument(0) {
            resolve_path(p)?
        } else {
            return Err(RuntimeError::InvalidOperation(
                "Set-Content requires a file path".to_string(),
            ));
        };

        // Value: -Value takes precedence, else pipeline input, else second positional argument
        let values: Vec<Value> = if let Some(v) = get_parameter_ci(&context, "Value") {
            normalize_value_to_lines(v.clone())
                .into_iter()
                .map(Value::String)
                .collect()
        } else if !context.pipeline_input.is_empty() {
            context.pipeline_input
        } else if let Some(v) = context.get_argument(1) {
            normalize_value_to_lines(v.clone())
                .into_iter()
                .map(Value::String)
                .collect()
        } else {
            return Err(RuntimeError::InvalidOperation(
                "Set-Content requires a value to write (use -Value or pipeline input)".to_string(),
            ));
        };

        let data = values_to_file_string(values);

        fs::write(&path, data).map_err(|e| {
            RuntimeError::InvalidOperation(format!(
                "Failed to write file '{}': {}",
                path.display(),
                e
            ))
        })?;

        // Set-Content does not emit pipeline output by default.
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_set_content_writes_string_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("out.txt");

        let cmdlet = SetContentCmdlet;
        let context = CmdletContext::new().with_arguments(vec![
            Value::String(file_path.to_string_lossy().to_string()),
            Value::String("Hello".to_string()),
        ]);
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator);
        assert!(result.is_ok());

        let contents = fs::read_to_string(&file_path).unwrap();
        assert_eq!(contents, "Hello\n");
    }

    #[test]
    fn test_set_content_writes_array_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("out.txt");

        let cmdlet = SetContentCmdlet;
        let context = CmdletContext::new()
            .with_parameter(
                "Path".to_string(),
                Value::String(file_path.to_string_lossy().to_string()),
            )
            .with_parameter(
                "Value".to_string(),
                Value::Array(vec![
                    Value::String("Line 1".to_string()),
                    Value::String("Line 2".to_string()),
                    Value::String("Line 3".to_string()),
                ]),
            );
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator);
        assert!(result.is_ok());

        let contents = fs::read_to_string(&file_path).unwrap();
        assert_eq!(contents, "Line 1\nLine 2\nLine 3\n");
    }

    #[test]
    fn test_set_content_overwrites_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("out.txt");
        fs::write(&file_path, "Old\n").unwrap();

        let cmdlet = SetContentCmdlet;
        let context = CmdletContext::new().with_arguments(vec![
            Value::String(file_path.to_string_lossy().to_string()),
            Value::String("New".to_string()),
        ]);
        let mut evaluator = pwsh_runtime::Evaluator::new();
        cmdlet.execute(context, &mut evaluator).unwrap();

        let contents = fs::read_to_string(&file_path).unwrap();
        assert_eq!(contents, "New\n");
    }

    #[test]
    fn test_set_content_pipeline_input_writes_lines() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("out.txt");

        let cmdlet = SetContentCmdlet;
        let context = CmdletContext::with_input(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ])
        .with_arguments(vec![Value::String(file_path.to_string_lossy().to_string())]);
        let mut evaluator = pwsh_runtime::Evaluator::new();
        cmdlet.execute(context, &mut evaluator).unwrap();

        let contents = fs::read_to_string(&file_path).unwrap();
        assert_eq!(contents, "a\nb\n");
    }

    #[test]
    fn test_set_content_missing_path_errors() {
        let cmdlet = SetContentCmdlet;
        let context = CmdletContext::new()
            .with_parameter("Value".to_string(), Value::String("Hello".to_string()));
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_content_missing_value_errors() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("out.txt");

        let cmdlet = SetContentCmdlet;
        let context = CmdletContext::new()
            .with_parameter(
                "Path".to_string(),
                Value::String(file_path.to_string_lossy().to_string()),
            )
            .with_arguments(vec![Value::String(file_path.to_string_lossy().to_string())]);

        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator);
        assert!(result.is_err());
    }
}
