/// Get-ChildItem cmdlet - lists files and directories in the file system
use pwsh_runtime::{Cmdlet, CmdletContext, RuntimeError, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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

fn parse_string_patterns(value: Option<&Value>) -> Vec<String> {
    match value {
        None => vec![],
        Some(Value::String(s)) => s
            .split(',')
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .map(|p| p.to_string())
            .collect(),
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .flat_map(|s| {
                s.split(',')
                    .map(|p| p.trim())
                    .filter(|p| !p.is_empty())
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
            })
            .collect(),
        Some(other) => vec![other.to_string()],
    }
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

fn wildcard_match_case_insensitive(pattern: &str, text: &str) -> bool {
    // Supports: '*' (0+ chars) and '?' (exactly 1 char)
    let p = pattern.as_bytes();
    let t = text.as_bytes();

    let mut pi: usize = 0;
    let mut ti: usize = 0;
    let mut star_idx: Option<usize> = None;
    let mut match_ti: usize = 0;

    while ti < t.len() {
        if pi < p.len() && (p[pi] == b'?' || p[pi].eq_ignore_ascii_case(&t[ti])) {
            pi += 1;
            ti += 1;
        } else if pi < p.len() && p[pi] == b'*' {
            star_idx = Some(pi);
            pi += 1;
            match_ti = ti;
        } else if let Some(si) = star_idx {
            pi = si + 1;
            match_ti += 1;
            ti = match_ti;
        } else {
            return false;
        }
    }

    while pi < p.len() && p[pi] == b'*' {
        pi += 1;
    }

    pi == p.len()
}

fn matches_any_pattern(name: &str, patterns: &[String]) -> bool {
    patterns
        .iter()
        .any(|p| wildcard_match_case_insensitive(p, name))
}

fn parse_switch(value: Option<&Value>) -> Result<bool, RuntimeError> {
    match value {
        None => Ok(false),
        Some(Value::Boolean(b)) => Ok(*b),
        Some(Value::Number(n)) => Ok(*n != 0.0),
        Some(Value::String(s)) => {
            let v = s.trim().to_ascii_lowercase();
            match v.as_str() {
                "true" | "t" | "1" | "yes" | "y" => Ok(true),
                "false" | "f" | "0" | "no" | "n" => Ok(false),
                _ => Err(RuntimeError::InvalidOperation(format!(
                    "Invalid boolean value: {}",
                    s
                ))),
            }
        }
        Some(other) => Err(RuntimeError::InvalidOperation(format!(
            "Invalid boolean value: {}",
            other
        ))),
    }
}

fn parse_optional_depth(value: Option<&Value>) -> Result<Option<usize>, RuntimeError> {
    match value {
        None => Ok(None),
        Some(Value::Number(n)) => {
            if *n < 0.0 {
                return Err(RuntimeError::InvalidOperation(
                    "Depth must be a non-negative number".to_string(),
                ));
            }
            Ok(Some(*n as usize))
        }
        Some(Value::String(s)) => {
            let parsed: usize = s.trim().parse().map_err(|_| {
                RuntimeError::InvalidOperation(format!("Depth must be an integer, got: {}", s))
            })?;
            Ok(Some(parsed))
        }
        Some(other) => Err(RuntimeError::InvalidOperation(format!(
            "Depth must be a number, got: {}",
            other
        ))),
    }
}

fn should_output_name(
    name: &str,
    filter_patterns: &[String],
    include_patterns: &[String],
    exclude_patterns: &[String],
) -> bool {
    if !filter_patterns.is_empty() && !matches_any_pattern(name, filter_patterns) {
        return false;
    }
    if !include_patterns.is_empty() && !matches_any_pattern(name, include_patterns) {
        return false;
    }
    if !exclude_patterns.is_empty() && matches_any_pattern(name, exclude_patterns) {
        return false;
    }
    true
}

fn collect_directory_items(
    root: &Path,
    recurse: bool,
    max_depth: Option<usize>,
    filter_patterns: &[String],
    include_patterns: &[String],
    exclude_patterns: &[String],
) -> Result<Vec<Value>, RuntimeError> {
    // Depth is measured in "directory hops" from the root.
    // depth=0 means: list only the root directory entries (no recursion).
    let mut items: Vec<Value> = Vec::new();
    let mut stack: Vec<(PathBuf, usize)> = vec![(root.to_path_buf(), 0)];

    while let Some((dir, depth)) = stack.pop() {
        let entries = fs::read_dir(&dir).map_err(|e| {
            RuntimeError::InvalidOperation(format!(
                "Failed to read directory '{}': {}",
                dir.display(),
                e
            ))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                RuntimeError::InvalidOperation(format!("Failed to read directory entry: {}", e))
            })?;

            let entry_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Decide recursion separately from output filtering:
            // we still want to traverse subdirectories even when filter/include don't match
            // directory names (e.g. -Recurse -Filter *.rs).
            if recurse {
                let within_depth = max_depth.map(|d| depth < d).unwrap_or(true);
                if within_depth {
                    let ft = fs::symlink_metadata(&entry_path)
                        .map(|m| m.file_type())
                        .map_err(|e| {
                            RuntimeError::InvalidOperation(format!(
                                "Failed to read metadata for '{}': {}",
                                entry_path.display(),
                                e
                            ))
                        })?;

                    let is_symlink = ft.is_symlink();
                    // Avoid infinite loops: do not recurse into symlinked directories.
                    if !is_symlink && entry_path.is_dir() {
                        stack.push((entry_path.clone(), depth + 1));
                    }
                }
            }

            if !should_output_name(&name, filter_patterns, include_patterns, exclude_patterns) {
                continue;
            }

            items.push(build_file_object(&entry_path, name)?);
        }
    }

    Ok(items)
}

fn build_mode_string(metadata: &fs::Metadata) -> String {
    // Cross-platform mode formatting.
    // - On Unix: use actual permission bits.
    // - Elsewhere (Windows): best-effort string based on readonly + directory.

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mode = metadata.permissions().mode();
        let is_dir = metadata.is_dir();
        let file_type = if is_dir { 'd' } else { '-' };

        let bits = [
            (0o400, 'r'),
            (0o200, 'w'),
            (0o100, 'x'),
            (0o040, 'r'),
            (0o020, 'w'),
            (0o010, 'x'),
            (0o004, 'r'),
            (0o002, 'w'),
            (0o001, 'x'),
        ];

        let mut s = String::with_capacity(10);
        s.push(file_type);
        for (mask, ch) in bits {
            s.push(if (mode & mask) != 0 { ch } else { '-' });
        }
        s
    }

    #[cfg(not(unix))]
    {
        let is_dir = metadata.is_dir();
        let readonly = metadata.permissions().readonly();

        // Best-effort POSIX-like mode string.
        // Examples:
        //  - Directory (writable): drwxr-xr-x
        //  - Directory (readonly): dr-xr-xr-x
        //  - File (writable):     -rw-r--r--
        //  - File (readonly):     -r--r--r--
        let file_type = if is_dir { 'd' } else { '-' };
        let owner_write = if readonly { '-' } else { 'w' };
        let owner_exec = if is_dir { 'x' } else { '-' };

        let owner = format!("r{}{}", owner_write, owner_exec);
        let group = format!("r-{}", if is_dir { 'x' } else { '-' });
        let other = format!("r-{}", if is_dir { 'x' } else { '-' });
        format!("{}{}{}{}", file_type, owner, group, other)
    }
}

fn system_time_to_unix_epoch_seconds(t: SystemTime) -> f64 {
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs_f64(),
        Err(_) => 0.0,
    }
}

fn build_file_object(path: &Path, name: String) -> Result<Value, RuntimeError> {
    let metadata = fs::metadata(path).map_err(|e| {
        RuntimeError::InvalidOperation(format!(
            "Failed to read metadata for '{}': {}",
            path.display(),
            e
        ))
    })?;

    let is_dir = metadata.is_dir();

    let length = if metadata.is_file() {
        metadata.len() as f64
    } else {
        0.0
    };
    let last_write_time = metadata
        .modified()
        .map(system_time_to_unix_epoch_seconds)
        .unwrap_or(0.0);
    let mode = build_mode_string(&metadata);

    let mut props = HashMap::with_capacity(5);
    props.insert("Name".to_string(), Value::String(name));
    props.insert("Length".to_string(), Value::Number(length));
    props.insert("LastWriteTime".to_string(), Value::Number(last_write_time));
    props.insert("Mode".to_string(), Value::String(mode));
    props.insert("Directory".to_string(), Value::Boolean(is_dir));

    Ok(Value::Object(props))
}

/// Get-ChildItem cmdlet retrieves child items (files/directories) in a location
pub struct GetChildItemCmdlet;

impl Cmdlet for GetChildItemCmdlet {
    fn name(&self) -> &str {
        "Get-ChildItem"
    }

    fn execute(
        &self,
        context: CmdletContext,
        _evaluator: &mut pwsh_runtime::Evaluator,
    ) -> Result<Vec<Value>, RuntimeError> {
        // Parameters
        let filter_patterns = parse_string_patterns(get_parameter_ci(&context, "Filter"));
        let include_patterns = parse_string_patterns(get_parameter_ci(&context, "Include"));
        let exclude_patterns = parse_string_patterns(get_parameter_ci(&context, "Exclude"));

        let recurse = parse_switch(get_parameter_ci(&context, "Recurse"))?;
        let max_depth = parse_optional_depth(get_parameter_ci(&context, "Depth"))?;

        // Get path from parameters or arguments, default to current directory
        let path = if let Some(Value::String(p)) = get_parameter_ci(&context, "Path") {
            resolve_path(p)?
        } else if let Some(Value::String(p)) = context.get_argument(0) {
            resolve_path(p)?
        } else {
            std::env::current_dir().map_err(|e| {
                RuntimeError::InvalidOperation(format!("Failed to get current directory: {}", e))
            })?
        };

        let metadata = fs::metadata(&path).map_err(|e| {
            RuntimeError::InvalidOperation(format!(
                "Failed to access path '{}': {}",
                path.display(),
                e
            ))
        })?;

        // If it's a file, return just that file as a single item.
        if metadata.is_file() {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string());

            // Apply name-based filters
            if !filter_patterns.is_empty() && !matches_any_pattern(&name, &filter_patterns) {
                return Ok(vec![]);
            }
            if !include_patterns.is_empty() && !matches_any_pattern(&name, &include_patterns) {
                return Ok(vec![]);
            }
            if !exclude_patterns.is_empty() && matches_any_pattern(&name, &exclude_patterns) {
                return Ok(vec![]);
            }

            return Ok(vec![build_file_object(&path, name)?]);
        }

        // Otherwise, read directory contents (optionally recursively)
        collect_directory_items(
            &path,
            recurse,
            max_depth,
            &filter_patterns,
            &include_patterns,
            &exclude_patterns,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_get_childitem_lists_files() {
        // Create a temporary directory with some files
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        File::create(temp_path.join("file1.txt")).unwrap();
        File::create(temp_path.join("file2.txt")).unwrap();
        File::create(temp_path.join("file3.rs")).unwrap();

        // Execute cmdlet with path as argument
        let cmdlet = GetChildItemCmdlet;
        let context = CmdletContext::new()
            .with_arguments(vec![Value::String(temp_path.to_string_lossy().to_string())]);
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        // Verify we have at least 3 files (temp dirs may have extra files)
        assert!(
            result.len() >= 3,
            "Should return at least 3 files, got {}",
            result.len()
        );

        // Verify that our test files are present
        let mut found_file1 = false;
        let mut found_file2 = false;
        let mut found_file3 = false;

        for item in &result {
            if let Value::Object(props) = item {
                assert!(
                    props.contains_key("Name"),
                    "Each item should have a Name property"
                );
                if let Some(Value::String(name)) = props.get("Name") {
                    if name == "file1.txt" {
                        found_file1 = true;
                    } else if name == "file2.txt" {
                        found_file2 = true;
                    } else if name == "file3.rs" {
                        found_file3 = true;
                    }
                } else {
                    panic!("Name property should be a string");
                }
            } else {
                panic!("Each item should be an Object");
            }
        }

        assert!(found_file1, "Should find file1.txt");
        assert!(found_file2, "Should find file2.txt");
        assert!(found_file3, "Should find file3.rs");
    }

    #[test]
    fn test_get_childitem_empty_directory() {
        // Create an empty temporary directory
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Execute cmdlet with path as argument
        let cmdlet = GetChildItemCmdlet;
        let context = CmdletContext::new()
            .with_arguments(vec![Value::String(temp_path.to_string_lossy().to_string())]);
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        // Verify results
        assert_eq!(result.len(), 0, "Should return 0 files for empty directory");
    }

    #[test]
    fn test_get_childitem_includes_directories() {
        // Create a temporary directory with files and subdirectories
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files and directories
        File::create(temp_path.join("file.txt")).unwrap();
        fs::create_dir(temp_path.join("subdir")).unwrap();

        // Execute cmdlet with path as argument
        let cmdlet = GetChildItemCmdlet;
        let context = CmdletContext::new()
            .with_arguments(vec![Value::String(temp_path.to_string_lossy().to_string())]);
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        // Verify results
        assert_eq!(result.len(), 2, "Should return both file and directory");

        // Verify each item has a Name property
        let mut found_file = false;
        let mut found_dir = false;
        for item in &result {
            if let Value::Object(props) = item {
                if let Some(Value::String(name)) = props.get("Name") {
                    if name == "file.txt" {
                        found_file = true;
                    } else if name == "subdir" {
                        found_dir = true;
                    }
                }
            }
        }
        assert!(found_file, "Should find file.txt");
        assert!(found_dir, "Should find subdir directory");
    }

    #[test]
    fn test_get_childitem_nonexistent_directory() {
        // Test error handling for non-existent directory
        let cmdlet = GetChildItemCmdlet;
        // Use an absolute path so resolve_path() doesn't accidentally treat Windows-style
        // paths as relative on Unix.
        let nonexistent_path = if cfg!(windows) {
            std::path::PathBuf::from(r"C:\definitely\nonexistent\directory\path")
        } else {
            std::path::PathBuf::from("/definitely/nonexistent/directory/path")
        };
        let context = CmdletContext::new().with_arguments(vec![Value::String(
            nonexistent_path.to_string_lossy().to_string(),
        )]);
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator);

        // Verify that it returns an error
        assert!(
            result.is_err(),
            "Should return error for non-existent directory"
        );

        // Verify the error message contains relevant information
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Failed to access path")
                    || error_msg.contains("Failed to read directory")
                    || error_msg.contains("The system cannot find the path specified")
                    || error_msg.contains("No such file or directory"),
                "Error message should indicate directory read failure: {}",
                error_msg
            );
        }
    }

    #[test]
    fn test_get_childitem_path_parameter_case_insensitive() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        File::create(temp_path.join("file1.txt")).unwrap();

        let cmdlet = GetChildItemCmdlet;
        let context = CmdletContext::new().with_parameter(
            "path".to_string(),
            Value::String(temp_path.to_string_lossy().to_string()),
        );
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_get_childitem_relative_path_resolution() {
        // Create a directory relative to the current working directory to validate
        // resolve_path() without mutating process current_dir (tests run in parallel).
        let cwd = std::env::current_dir().unwrap();
        let unique = format!(
            "pwsh_get_childitem_rel_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let rel_dir = PathBuf::from(&unique);
        let abs_dir = cwd.join(&rel_dir);
        fs::create_dir_all(&abs_dir).unwrap();
        File::create(abs_dir.join("file1.txt")).unwrap();

        let cmdlet = GetChildItemCmdlet;
        let context = CmdletContext::new().with_parameter(
            "Path".to_string(),
            Value::String(rel_dir.to_string_lossy().to_string()),
        );
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        // Cleanup best-effort
        let _ = fs::remove_file(abs_dir.join("file1.txt"));
        let _ = fs::remove_dir_all(&abs_dir);

        assert_eq!(result.len(), 1);
        if let Value::Object(props) = &result[0] {
            assert_eq!(
                props.get("Name"),
                Some(&Value::String("file1.txt".to_string()))
            );
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_get_childitem_rich_properties_present() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        File::create(temp_path.join("file1.txt")).unwrap();
        fs::create_dir(temp_path.join("subdir")).unwrap();

        let cmdlet = GetChildItemCmdlet;
        let context = CmdletContext::new()
            .with_arguments(vec![Value::String(temp_path.to_string_lossy().to_string())]);
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result.len(), 2);

        for item in &result {
            if let Value::Object(props) = item {
                assert!(props.contains_key("Name"));
                assert!(props.contains_key("Length"));
                assert!(props.contains_key("LastWriteTime"));
                assert!(props.contains_key("Mode"));
                assert!(props.contains_key("Directory"));

                assert!(matches!(props.get("Name"), Some(Value::String(_))));
                assert!(matches!(props.get("Length"), Some(Value::Number(_))));
                assert!(matches!(props.get("LastWriteTime"), Some(Value::Number(_))));
                assert!(matches!(props.get("Mode"), Some(Value::String(_))));
                assert!(matches!(props.get("Directory"), Some(Value::Boolean(_))));
            } else {
                panic!("Expected object");
            }
        }
    }

    #[test]
    fn test_get_childitem_mode_string_format() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        File::create(temp_path.join("file1.txt")).unwrap();

        let cmdlet = GetChildItemCmdlet;
        let context = CmdletContext::new()
            .with_arguments(vec![Value::String(temp_path.to_string_lossy().to_string())]);
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        assert_eq!(result.len(), 1);
        if let Value::Object(props) = &result[0] {
            let mode = match props.get("Mode") {
                Some(Value::String(s)) => s,
                _ => panic!("Expected Mode to be a string"),
            };
            assert_eq!(mode.len(), 10, "Mode string should be 10 chars");
            let first = mode.chars().next().unwrap();
            assert!(first == '-' || first == 'd');
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_get_childitem_filter_include_exclude() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        File::create(temp_path.join("file1.txt")).unwrap();
        File::create(temp_path.join("file2.rs")).unwrap();
        File::create(temp_path.join("README.md")).unwrap();

        let cmdlet = GetChildItemCmdlet;
        let mut evaluator = pwsh_runtime::Evaluator::new();

        // -Filter
        let context = CmdletContext::new()
            .with_parameter(
                "Path".to_string(),
                Value::String(temp_path.to_string_lossy().to_string()),
            )
            .with_parameter("Filter".to_string(), Value::String("*.rs".to_string()));
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result.len(), 1);
        if let Value::Object(props) = &result[0] {
            assert_eq!(
                props.get("Name"),
                Some(&Value::String("file2.rs".to_string()))
            );
        } else {
            panic!("Expected object");
        }

        // -Include/-Exclude
        let context = CmdletContext::new()
            .with_parameter(
                "Path".to_string(),
                Value::String(temp_path.to_string_lossy().to_string()),
            )
            .with_parameter(
                "Include".to_string(),
                Value::Array(vec![
                    Value::String("*.md".to_string()),
                    Value::String("*.txt".to_string()),
                ]),
            )
            .with_parameter("Exclude".to_string(), Value::String("README*".to_string()));
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result.len(), 1);
        if let Value::Object(props) = &result[0] {
            assert_eq!(
                props.get("Name"),
                Some(&Value::String("file1.txt".to_string()))
            );
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_get_childitem_recurse_lists_nested_items() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::create_dir(root.join("subdir")).unwrap();
        File::create(root.join("root.txt")).unwrap();
        File::create(root.join("subdir").join("nested.txt")).unwrap();

        let cmdlet = GetChildItemCmdlet;
        let context = CmdletContext::new()
            .with_parameter("Path".to_string(), Value::String(root.to_string_lossy().to_string()))
            .with_parameter("Recurse".to_string(), Value::Boolean(true));
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        let mut found_root = false;
        let mut found_nested = false;
        for item in &result {
            if let Value::Object(props) = item {
                if let Some(Value::String(name)) = props.get("Name") {
                    if name == "root.txt" {
                        found_root = true;
                    } else if name == "nested.txt" {
                        found_nested = true;
                    }
                }
            }
        }

        assert!(found_root, "Should find root.txt");
        assert!(found_nested, "Should find nested.txt when recursing");
    }

    #[test]
    fn test_get_childitem_recurse_depth_limits_traversal() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::create_dir_all(root.join("a").join("b")).unwrap();
        File::create(root.join("a").join("depth1.txt")).unwrap();
        File::create(root.join("a").join("b").join("depth2.txt")).unwrap();

        let cmdlet = GetChildItemCmdlet;
        let context = CmdletContext::new()
            .with_parameter("Path".to_string(), Value::String(root.to_string_lossy().to_string()))
            .with_parameter("Recurse".to_string(), Value::Boolean(true))
            .with_parameter("Depth".to_string(), Value::Number(1.0));
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        let mut found_depth1 = false;
        let mut found_depth2 = false;
        for item in &result {
            if let Value::Object(props) = item {
                if let Some(Value::String(name)) = props.get("Name") {
                    if name == "depth1.txt" {
                        found_depth1 = true;
                    } else if name == "depth2.txt" {
                        found_depth2 = true;
                    }
                }
            }
        }

        assert!(found_depth1, "Should find depth1.txt at depth 1");
        assert!(
            !found_depth2,
            "Should not find depth2.txt when Depth is limited to 1"
        );
    }

    #[test]
    fn test_get_childitem_recurse_does_not_follow_symlink_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::create_dir_all(root.join("real")).unwrap();
        File::create(root.join("real").join("inside.txt")).unwrap();

        // Create a symlink that points back to root (potential loop). This is best-effort:
        // some platforms/permissions may not allow creating symlinks.
        let link_path = root.join("link");

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            if symlink(root, &link_path).is_err() {
                return; // Skip if symlinks aren't available
            }
        }

        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_dir;
            if symlink_dir(root, &link_path).is_err() {
                return; // Skip if symlinks aren't available
            }
        }

        let cmdlet = GetChildItemCmdlet;
        let context = CmdletContext::new()
            .with_parameter("Path".to_string(), Value::String(root.to_string_lossy().to_string()))
            .with_parameter("Recurse".to_string(), Value::Boolean(true))
            .with_parameter("Depth".to_string(), Value::Number(5.0));
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        // If we mistakenly follow the symlink loop, this test would likely hang or explode.
        // We also verify we still find the real nested file.
        let mut found_inside = false;
        for item in &result {
            if let Value::Object(props) = item {
                if let Some(Value::String(name)) = props.get("Name") {
                    if name == "inside.txt" {
                        found_inside = true;
                    }
                }
            }
        }
        assert!(found_inside, "Should find nested file inside real directory");
    }
}
