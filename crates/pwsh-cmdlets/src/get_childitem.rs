/// Get-ChildItem cmdlet - lists files and directories in the file system
use pwsh_runtime::{Cmdlet, CmdletContext, RuntimeError, Value};
use std::collections::HashMap;
use std::fs;

/// Get-ChildItem cmdlet retrieves child items (files/directories) in a location
pub struct GetChildItemCmdlet;

impl Cmdlet for GetChildItemCmdlet {
    fn name(&self) -> &str {
        "Get-ChildItem"
    }

    fn execute(
        &self,
        _context: CmdletContext,
        _evaluator: &mut pwsh_runtime::Evaluator,
    ) -> Result<Vec<Value>, RuntimeError> {
        // Get current directory
        let current_dir = std::env::current_dir().map_err(|e| {
            RuntimeError::InvalidOperation(format!("Failed to get current directory: {}", e))
        })?;

        // Read directory contents
        let entries = fs::read_dir(&current_dir).map_err(|e| {
            RuntimeError::InvalidOperation(format!(
                "Failed to read directory '{}': {}",
                current_dir.display(),
                e
            ))
        })?;

        // Collect file/directory objects
        let mut items = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| {
                RuntimeError::InvalidOperation(format!("Failed to read directory entry: {}", e))
            })?;

            let file_name = entry.file_name();
            let name = file_name.to_string_lossy().to_string();

            // Create file object with Name property
            let mut props = HashMap::new();
            props.insert("Name".to_string(), Value::String(name));

            items.push(Value::Object(props));
        }

        Ok(items)
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

        // Change to temp directory and run test
        let original_dir = std::env::current_dir().unwrap();
        let result = std::panic::catch_unwind(|| {
            std::env::set_current_dir(temp_path).unwrap();

            // Execute cmdlet
            let cmdlet = GetChildItemCmdlet;
            let context = CmdletContext::new();
            let mut evaluator = pwsh_runtime::Evaluator::new();
            cmdlet.execute(context, &mut evaluator).unwrap()
        });

        // Restore original directory before checking results
        std::env::set_current_dir(original_dir).unwrap();

        let result = result.unwrap();

        // Verify we have at least 3 files (temp dirs may have extra files)
        assert!(result.len() >= 3, "Should return at least 3 files, got {}", result.len());

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

        // Change to temp directory and run test
        let original_dir = std::env::current_dir().unwrap();
        let result = std::panic::catch_unwind(|| {
            std::env::set_current_dir(temp_path).unwrap();

            // Execute cmdlet
            let cmdlet = GetChildItemCmdlet;
            let context = CmdletContext::new();
            let mut evaluator = pwsh_runtime::Evaluator::new();
            cmdlet.execute(context, &mut evaluator).unwrap()
        });

        // Restore original directory before checking results
        std::env::set_current_dir(original_dir).unwrap();

        let result = result.unwrap();

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

        // Change to temp directory and run test
        let original_dir = std::env::current_dir().unwrap();
        let result = std::panic::catch_unwind(|| {
            std::env::set_current_dir(temp_path).unwrap();

            // Execute cmdlet
            let cmdlet = GetChildItemCmdlet;
            let context = CmdletContext::new();
            let mut evaluator = pwsh_runtime::Evaluator::new();
            cmdlet.execute(context, &mut evaluator).unwrap()
        });

        // Restore original directory before checking results
        std::env::set_current_dir(original_dir).unwrap();

        let result = result.unwrap();

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
}
