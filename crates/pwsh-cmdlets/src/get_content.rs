/// Get-Content cmdlet - reads a file and returns its contents as an array of strings (one per line)
use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;
use pwsh_runtime::{Cmdlet, CmdletContext, RuntimeError, Value};
use std::collections::VecDeque;
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

fn parse_encoding(value: Option<&Value>) -> Result<Option<&'static Encoding>, RuntimeError> {
    let Some(v) = value else {
        return Ok(None);
    };

    let s = match v {
        Value::String(s) => s.as_str(),
        other => {
            return Err(RuntimeError::InvalidOperation(format!(
                "Encoding must be a string, got: {}",
                other
            )))
        }
    };

    let enc = match s.trim().to_ascii_lowercase().as_str() {
        // UTF-8
        "utf8" | "utf-8" | "utf8bom" => Some(encoding_rs::UTF_8),

        // ASCII (7-bit)
        "ascii" | "us-ascii" => Encoding::for_label(b"us-ascii"),

        // PowerShell naming: 'Unicode' == UTF-16LE
        "unicode" | "utf16" | "utf-16" | "utf-16le" => Some(encoding_rs::UTF_16LE),
        "bigendianunicode" | "utf-16be" => Some(encoding_rs::UTF_16BE),

        // Not supported by encoding_rs
        "utf32" | "utf-32" | "utf-32le" | "utf-32be" => {
            return Err(RuntimeError::InvalidOperation(
                "Unsupported encoding: UTF-32".to_string(),
            ));
        }

        "" => None,
        other => {
            return Err(RuntimeError::InvalidOperation(format!(
                "Unsupported encoding: {}",
                other
            )))
        }
    };

    Ok(enc)
}

fn parse_count_param(context: &CmdletContext, name: &str) -> Result<Option<usize>, RuntimeError> {
    let Some(v) = get_parameter_ci(context, name) else {
        return Ok(None);
    };

    let n = match v {
        Value::Number(n) => *n,
        Value::String(s) => s.trim().parse::<f64>().map_err(|_| {
            RuntimeError::InvalidOperation(format!(
                "{name} must be a non-negative integer, got: {s}"
            ))
        })?,
        other => {
            return Err(RuntimeError::InvalidOperation(format!(
                "{name} must be a non-negative integer, got: {other}"
            )))
        }
    };

    if n.is_nan() || n.is_infinite() || n < 0.0 || n.fract() != 0.0 {
        return Err(RuntimeError::InvalidOperation(format!(
            "{name} must be a non-negative integer, got: {n}"
        )));
    }

    if n > (usize::MAX as f64) {
        return Err(RuntimeError::InvalidOperation(format!(
            "{name} is too large: {n}"
        )));
    }

    Ok(Some(n as usize))
}

fn read_lines_filtered(
    path: &Path,
    encoding: Option<&'static Encoding>,
    total_count: Option<usize>,
    tail: Option<usize>,
) -> Result<Vec<Value>, RuntimeError> {
    if total_count.is_some() && tail.is_some() {
        return Err(RuntimeError::InvalidOperation(
            "Get-Content does not support using -TotalCount and -Tail together".to_string(),
        ));
    }

    let file = File::open(path).map_err(|e| {
        RuntimeError::InvalidOperation(format!("Failed to open file '{}': {}", path.display(), e))
    })?;

    // Stream-decoding reader:
    // - If -Encoding is provided, use it.
    // - Always BOM sniff so UTF-8/UTF-16 files with BOM read correctly.
    let mut builder = DecodeReaderBytesBuilder::new();
    builder.bom_sniffing(true);
    if let Some(enc) = encoding {
        builder.encoding(Some(enc));
    }

    let decoded = builder.build(file);
    let reader = BufReader::new(decoded);

    if total_count == Some(0) || tail == Some(0) {
        return Ok(vec![]);
    }

    // -Tail N: keep only the last N lines
    if let Some(tail) = tail {
        let mut buf: VecDeque<Value> = VecDeque::with_capacity(tail.max(1));

        for line in reader.lines() {
            let line = line.map_err(|e| {
                RuntimeError::InvalidOperation(format!(
                    "Failed to read file '{}': {}",
                    path.display(),
                    e
                ))
            })?;

            buf.push_back(Value::String(line));
            if buf.len() > tail {
                buf.pop_front();
            }
        }

        return Ok(buf.into_iter().collect());
    }

    // Default / -TotalCount: collect (streaming) and optionally stop early
    let mut out = Vec::new();
    let max_take = total_count.unwrap_or(usize::MAX);

    for (taken, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| {
            RuntimeError::InvalidOperation(format!(
                "Failed to read file '{}': {}",
                path.display(),
                e
            ))
        })?;

        if taken >= max_take {
            break;
        }

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
        let encoding = parse_encoding(get_parameter_ci(&context, "Encoding"))?;

        // Align with native PowerShell:
        // -TotalCount N (first N lines)
        // -Tail N (last N lines)
        let total_count = parse_count_param(&context, "TotalCount")?;
        let tail = parse_count_param(&context, "Tail")?;

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

        read_lines_filtered(&path, encoding, total_count, tail)
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
        let context = CmdletContext::new().with_parameter(
            "Path".to_string(),
            Value::String(file_path.to_string_lossy().to_string()),
        );
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_get_content_nonexistent_file_errors() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("does_not_exist.txt");

        let cmdlet = GetContentCmdlet;
        let context = CmdletContext::new().with_parameter(
            "Path".to_string(),
            Value::String(file_path.to_string_lossy().to_string()),
        );
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator);

        assert!(result.is_err());
        let msg = result.err().unwrap().to_string();
        assert!(msg.contains("Failed to open file") || msg.contains("Failed to read file"));
    }

    #[test]
    fn test_get_content_reads_utf16le_with_bom() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("utf16.txt");

        let s = "one\ntwo\n";
        let mut bytes: Vec<u8> = vec![0xFF, 0xFE]; // UTF-16LE BOM
        for u in s.encode_utf16() {
            bytes.push((u & 0x00FF) as u8);
            bytes.push((u >> 8) as u8);
        }
        fs::write(&file_path, bytes).unwrap();

        let cmdlet = GetContentCmdlet;
        let context = CmdletContext::new()
            .with_parameter("Encoding".to_string(), Value::String("Unicode".to_string()))
            .with_arguments(vec![Value::String(file_path.to_string_lossy().to_string())]);
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        assert_eq!(
            result,
            vec![
                Value::String("one".to_string()),
                Value::String("two".to_string())
            ]
        );
    }

    #[test]
    fn test_get_content_invalid_encoding_errors() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("sample.txt");
        fs::write(&file_path, "hello\n").unwrap();

        let cmdlet = GetContentCmdlet;
        let context = CmdletContext::new()
            .with_parameter("Encoding".to_string(), Value::String("utf32".to_string()))
            .with_parameter(
                "Path".to_string(),
                Value::String(file_path.to_string_lossy().to_string()),
            );
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator);

        assert!(result.is_err());
        let msg = result.err().unwrap().to_string();
        assert!(msg.to_ascii_lowercase().contains("unsupported encoding"));
    }

    #[test]
    fn test_get_content_total_count() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("lines.txt");
        fs::write(&file_path, "one\ntwo\nthree\nfour\nfive\n").unwrap();

        let cmdlet = GetContentCmdlet;
        let context = CmdletContext::new()
            .with_parameter(
                "Path".to_string(),
                Value::String(file_path.to_string_lossy().to_string()),
            )
            .with_parameter("TotalCount".to_string(), Value::Number(2.0));
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        assert_eq!(
            result,
            vec![
                Value::String("one".to_string()),
                Value::String("two".to_string())
            ]
        );
    }

    #[test]
    fn test_get_content_tail() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("lines.txt");
        fs::write(&file_path, "one\ntwo\nthree\nfour\nfive\n").unwrap();

        let cmdlet = GetContentCmdlet;
        let context = CmdletContext::new()
            .with_parameter(
                "Path".to_string(),
                Value::String(file_path.to_string_lossy().to_string()),
            )
            .with_parameter("Tail".to_string(), Value::Number(2.0));
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        assert_eq!(
            result,
            vec![
                Value::String("four".to_string()),
                Value::String("five".to_string())
            ]
        );
    }

    #[test]
    fn test_get_content_total_count_and_tail_error() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("lines.txt");
        fs::write(&file_path, "one\ntwo\nthree\n").unwrap();

        let cmdlet = GetContentCmdlet;
        let context = CmdletContext::new()
            .with_parameter(
                "Path".to_string(),
                Value::String(file_path.to_string_lossy().to_string()),
            )
            .with_parameter("TotalCount".to_string(), Value::Number(1.0))
            .with_parameter("Tail".to_string(), Value::Number(1.0));
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator);

        assert!(result.is_err());
        let msg = result.err().unwrap().to_string().to_ascii_lowercase();
        assert!(msg.contains("totalcount") && msg.contains("tail"));
    }

    #[test]
    fn test_get_content_negative_total_count_errors() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("lines.txt");
        fs::write(&file_path, "one\ntwo\n").unwrap();

        let cmdlet = GetContentCmdlet;
        let context = CmdletContext::new()
            .with_parameter(
                "Path".to_string(),
                Value::String(file_path.to_string_lossy().to_string()),
            )
            .with_parameter("TotalCount".to_string(), Value::Number(-1.0));
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator);

        assert!(result.is_err());
        let msg = result.err().unwrap().to_string().to_ascii_lowercase();
        assert!(msg.contains("totalcount") && msg.contains("non-negative"));
    }

    #[test]
    fn test_get_content_fractional_tail_errors() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("lines.txt");
        fs::write(&file_path, "one\ntwo\n").unwrap();

        let cmdlet = GetContentCmdlet;
        let context = CmdletContext::new()
            .with_parameter(
                "Path".to_string(),
                Value::String(file_path.to_string_lossy().to_string()),
            )
            .with_parameter("Tail".to_string(), Value::Number(1.5));
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator);

        assert!(result.is_err());
        let msg = result.err().unwrap().to_string().to_ascii_lowercase();
        assert!(msg.contains("tail") && msg.contains("non-negative"));
    }
}
