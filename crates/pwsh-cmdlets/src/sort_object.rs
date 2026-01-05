/// Sort-Object cmdlet - sort pipeline objects by value or property
use pwsh_runtime::{Cmdlet, CmdletContext, RuntimeError, Value};

fn get_parameter_ci<'a>(context: &'a CmdletContext, name: &str) -> Option<&'a Value> {
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

fn parse_property_list(value: Option<&Value>) -> Vec<String> {
    match value {
        None => vec![],
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .collect(),
        Some(other) => vec![other.to_string()],
    }
}

fn unroll_to_items(values: Vec<Value>) -> Vec<Value> {
    let mut out = Vec::new();
    for v in values {
        if let Value::Array(items) = v {
            out.extend(items);
        } else {
            out.push(v);
        }
    }
    out
}

fn cmp_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    // Nulls sort first
    if matches!(a, Value::Null) && matches!(b, Value::Null) {
        return std::cmp::Ordering::Equal;
    }
    if matches!(a, Value::Null) {
        return std::cmp::Ordering::Less;
    }
    if matches!(b, Value::Null) {
        return std::cmp::Ordering::Greater;
    }

    // Prefer numeric comparison when both sides can be treated as numbers.
    let an = a.to_number();
    let bn = b.to_number();
    if let (Some(an), Some(bn)) = (an, bn) {
        return an
            .partial_cmp(&bn)
            .unwrap_or(std::cmp::Ordering::Equal);
    }

    // Fall back to case-insensitive string comparison (PowerShell default).
    a.to_string()
        .to_ascii_lowercase()
        .cmp(&b.to_string().to_ascii_lowercase())
}

/// Sort-Object sorts values/objects by one or more properties.
pub struct SortObjectCmdlet;

impl Cmdlet for SortObjectCmdlet {
    fn name(&self) -> &str {
        "Sort-Object"
    }

    fn execute(
        &self,
        context: CmdletContext,
        _evaluator: &mut pwsh_runtime::Evaluator,
    ) -> Result<Vec<Value>, RuntimeError> {
        let descending = parse_switch(get_parameter_ci(&context, "Descending"))?;

        let pipeline_has_input = !context.pipeline_input.is_empty();

        // Determine property list *before* moving vectors out of context.
        let mut properties = parse_property_list(get_parameter_ci(&context, "Property"));

        if pipeline_has_input && properties.is_empty() {
            // Support: $items | Sort-Object Name, CPU
            // Only treat positional args as property names when there is pipeline input.
            let positional_props = context
                .arguments
                .iter()
                .filter_map(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>();

            if !positional_props.is_empty() {
                properties = positional_props;
            }
        }

        // Determine input:
        // - When called in a pipeline, we sort pipeline input.
        // - When called standalone (no pipeline input), we sort the positional args as values.
        let mut input = if pipeline_has_input {
            context.pipeline_input
        } else {
            unroll_to_items(context.arguments)
        };

        // Sort in-place for performance.
        if properties.is_empty() {
            input.sort_by(|a, b| {
                let ord = cmp_values(a, b);
                if descending {
                    ord.reverse()
                } else {
                    ord
                }
            });

            return Ok(input);
        }

        input.sort_by(|a, b| {
            for prop in &properties {
                let av = a.get_property(prop).unwrap_or(Value::Null);
                let bv = b.get_property(prop).unwrap_or(Value::Null);
                let ord = cmp_values(&av, &bv);
                if ord != std::cmp::Ordering::Equal {
                    return if descending { ord.reverse() } else { ord };
                }
            }
            std::cmp::Ordering::Equal
        });

        Ok(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_sort_object_numbers_ascending() {
        let cmdlet = SortObjectCmdlet;
        let input = vec![Value::Number(3.0), Value::Number(1.0), Value::Number(2.0)];
        let context = CmdletContext::with_input(input);
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result, vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]);
    }

    #[test]
    fn test_sort_object_strings_case_insensitive() {
        let cmdlet = SortObjectCmdlet;
        let input = vec![
            Value::String("b".to_string()),
            Value::String("A".to_string()),
            Value::String("c".to_string()),
        ];
        let context = CmdletContext::with_input(input);
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(
            result,
            vec![
                Value::String("A".to_string()),
                Value::String("b".to_string()),
                Value::String("c".to_string())
            ]
        );
    }

    #[test]
    fn test_sort_object_by_property() {
        let cmdlet = SortObjectCmdlet;

        let mut o1 = HashMap::new();
        o1.insert("Name".to_string(), Value::String("b".to_string()));
        o1.insert("CPU".to_string(), Value::Number(2.0));

        let mut o2 = HashMap::new();
        o2.insert("Name".to_string(), Value::String("a".to_string()));
        o2.insert("CPU".to_string(), Value::Number(1.0));

        let context = CmdletContext::with_input(vec![Value::Object(o1), Value::Object(o2)])
            .with_parameter("Property".to_string(), Value::String("CPU".to_string()));

        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        if let Value::Object(props) = &result[0] {
            assert_eq!(props.get("Name"), Some(&Value::String("a".to_string())));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_sort_object_descending() {
        let cmdlet = SortObjectCmdlet;
        let input = vec![Value::Number(1.0), Value::Number(3.0), Value::Number(2.0)];
        let context = CmdletContext::with_input(input)
            .with_parameter("Descending".to_string(), Value::Boolean(true));
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result, vec![Value::Number(3.0), Value::Number(2.0), Value::Number(1.0)]);
    }

    #[test]
    fn test_sort_object_multiple_properties() {
        let cmdlet = SortObjectCmdlet;

        let mut o1 = HashMap::new();
        o1.insert("Name".to_string(), Value::String("b".to_string()));
        o1.insert("CPU".to_string(), Value::Number(1.0));

        let mut o2 = HashMap::new();
        o2.insert("Name".to_string(), Value::String("a".to_string()));
        o2.insert("CPU".to_string(), Value::Number(1.0));

        let mut o3 = HashMap::new();
        o3.insert("Name".to_string(), Value::String("c".to_string()));
        o3.insert("CPU".to_string(), Value::Number(0.0));

        let context = CmdletContext::with_input(vec![Value::Object(o1), Value::Object(o2), Value::Object(o3)])
            .with_parameter(
                "Property".to_string(),
                Value::Array(vec![Value::String("CPU".to_string()), Value::String("Name".to_string())]),
            );

        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        // Expect CPU=0 first, then CPU=1 sorted by Name (a then b)
        let names: Vec<String> = result
            .iter()
            .filter_map(|v| v.get_property("Name"))
            .map(|v| v.to_string())
            .collect();
        assert_eq!(names, vec!["c".to_string(), "a".to_string(), "b".to_string()]);
    }
}
