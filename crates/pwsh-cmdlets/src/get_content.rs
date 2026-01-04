/// Get-Content cmdlet - reads a file and returns its contents as an array of strings (one per line)
use pwsh_runtime::{Cmdlet, CmdletContext, RuntimeError, Value};
use std::fs::File;
use std::io::{BufRead, BufReader};
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

fn read_lines(path: &Path) -> Result<Vec<Value>, RuntimeError> {
    let file = File::open(path).map_err(|e| {
        RuntimeError::InvalidOperation(format!(
            "Failed to open file '{}': {}",
            path.display(),
            e
        ))
    })?;

    let reader = BufReader::new(file);
    let mut out = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|e| {
            RuntimeError::InvalidOperation(format!(
                "Failed to read file '{}': {}",
                path.display(),
                e
            ))
        })?;
        out.push(Value::String(line));
    }

    Ok(out)
}

/// Get-Content cmdlet reads file contents
pub struct GetContentCmdlet;

impl Cmdlet for GetContentCmdlet {
    fn name(&self) -> &str {
        "Get-Content"
    }

    fn execute(
        &self,
        context: CmdletContext,
        _evaluator: &mut pwsh_runtime::Evaluator,
    ) -> Result<Vec<Value>, RuntimeError> {
        // Get path from parameters or arguments
        let path = if let Some(Value::String(p)) = get_parameter_ci(&context, "Path") {
            resolve_path(p)?
        } else if let Some(Value::String(p)) = context.get_argument(0) {
            resolve_path(p)?
        } else {
            return Err(RuntimeError::InvalidOperation(
                "Get-Content requires a file path".to_string(),
            ));
        };

        read_lines(&path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_get_content_reads_text_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("sample.txt");
        fs::write(&file_path, "one\ntwo\nthree\n").unwrap();

        let cmdlet = GetContentCmdlet;
        let context = CmdletContext::new()
            .with_arguments(vec![Value::String(file_path.to_string_lossy().to_string())]);
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        assert_eq!(
            result,
            vec![
                Value::String("one".to_string()),
                Value::String("two".to_string()),
                Value::String("three".to_string())
            ]
        );
    }

    #[test]
    fn test_get_content_reads_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.txt");
        fs::write(&file_path, "").unwrap();

        let cmdlet = GetContentCmdlet;
        let context = CmdletContext::new()
            .with_parameter("Path".to_string(), Value::String(file_path.to_string_lossy().to_string()));
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_get_content_nonexistent_file_errors() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("does_not_exist.txt");

        let cmdlet = GetContentCmdlet;
        let context = CmdletContext::new()
            .with_parameter("Path".to_string(), Value::String(file_path.to_string_lossy().to_string()));
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator);

        assert!(result.is_err());
        let msg = result.err().unwrap().to_string();
        assert!(msg.contains("Failed to open file") || msg.contains("Failed to read file"));
    }
}
