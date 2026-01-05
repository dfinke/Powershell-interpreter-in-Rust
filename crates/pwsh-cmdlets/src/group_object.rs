/// Group-Object cmdlet - group pipeline objects by value or property
use pwsh_runtime::{Cmdlet, CmdletContext, RuntimeError, Value};
use std::collections::{BTreeMap, HashMap};

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

fn group_key_for_item(item: &Value, properties: &[String]) -> String {
    if properties.is_empty() {
        return item.to_string();
    }

    let mut parts = Vec::with_capacity(properties.len());
    for prop in properties {
        let v = item.get_property(prop).unwrap_or(Value::Null);
        parts.push(v.to_string());
    }

    // Keep it simple and deterministic; PowerShell uses a complex key for multi-property groups.
    // This string key is sufficient for our Value model and tests.
    parts.join(",")
}

fn build_group_info(name: String, group: Vec<Value>, no_element: bool) -> Value {
    let mut props = HashMap::new();
    props.insert("Count".to_string(), Value::Number(group.len() as f64));
    props.insert("Name".to_string(), Value::String(name));
    if !no_element {
        props.insert("Group".to_string(), Value::Array(group));
    }
    Value::Object(props)
}

/// Group-Object groups values/objects by one or more properties.
pub struct GroupObjectCmdlet;

impl Cmdlet for GroupObjectCmdlet {
    fn name(&self) -> &str {
        "Group-Object"
    }

    fn execute(
        &self,
        context: CmdletContext,
        _evaluator: &mut pwsh_runtime::Evaluator,
    ) -> Result<Vec<Value>, RuntimeError> {
        let no_element = parse_switch(get_parameter_ci(&context, "NoElement"))?;
        let as_hash_table = parse_switch(get_parameter_ci(&context, "AsHashTable"))?;

        let mut properties = parse_property_list(get_parameter_ci(&context, "Property"));

        // Input and positional-property behavior mirrors Sort-Object.
        let input = if !context.pipeline_input.is_empty() {
            if properties.is_empty() {
                // Support: $items | Group-Object Name, Extension
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
            context.pipeline_input
        } else {
            // Standalone: Group-Object 1 2 2 3
            unroll_to_items(context.arguments)
        };

        // Group deterministically using a BTreeMap (sorted by key).
        let mut groups: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        for item in input {
            let key = group_key_for_item(&item, &properties);
            groups.entry(key).or_default().push(item);
        }

        if as_hash_table {
            // Return a single hashtable-like object mapping group name -> GroupInfo.
            let mut map = HashMap::new();
            for (k, v) in groups {
                map.insert(k.clone(), build_group_info(k, v, no_element));
            }
            return Ok(vec![Value::Object(map)]);
        }

        let mut output = Vec::new();
        for (k, v) in groups {
            output.push(build_group_info(k, v, no_element));
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_object_numbers() {
        let cmdlet = GroupObjectCmdlet;
        let input = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(3.0),
            Value::Number(3.0),
        ];
        let context = CmdletContext::with_input(input);
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        assert_eq!(result.len(), 3);

        // Group keys should be "1", "2", "3" in sorted order.
        let names: Vec<String> = result
            .iter()
            .filter_map(|v| v.get_property("Name"))
            .map(|v| v.to_string())
            .collect();
        assert_eq!(names, vec!["1".to_string(), "2".to_string(), "3".to_string()]);

        let counts: Vec<f64> = result
            .iter()
            .filter_map(|v| v.get_property("Count"))
            .filter_map(|v| v.to_number())
            .collect();
        assert_eq!(counts, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_group_object_no_element() {
        let cmdlet = GroupObjectCmdlet;
        let input = vec![Value::Number(1.0), Value::Number(1.0)];
        let context = CmdletContext::with_input(input)
            .with_parameter("NoElement".to_string(), Value::Boolean(true));
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        assert_eq!(result.len(), 1);
        assert!(result[0].get_property("Group").is_none());
        assert_eq!(result[0].get_property("Count"), Some(Value::Number(2.0)));
    }

    #[test]
    fn test_group_object_as_hash_table() {
        let cmdlet = GroupObjectCmdlet;
        let input = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(2.0)];
        let context = CmdletContext::with_input(input)
            .with_parameter("AsHashTable".to_string(), Value::Boolean(true));
        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();

        assert_eq!(result.len(), 1);
        let Value::Object(map) = &result[0] else {
            panic!("Expected object hashtable result");
        };

        assert!(map.contains_key("1"));
        assert!(map.contains_key("2"));

        let gi2 = map.get("2").unwrap();
        assert_eq!(gi2.get_property("Count"), Some(Value::Number(2.0)));
    }

    #[test]
    fn test_group_object_by_property() {
        let cmdlet = GroupObjectCmdlet;

        let a = Value::Object(HashMap::from([
            (
                "Extension".to_string(),
                Value::String(".rs".to_string()),
            ),
            ("Name".to_string(), Value::String("a".to_string())),
        ]));
        let b = Value::Object(HashMap::from([
            (
                "Extension".to_string(),
                Value::String(".rs".to_string()),
            ),
            ("Name".to_string(), Value::String("b".to_string())),
        ]));
        let c = Value::Object(HashMap::from([
            (
                "Extension".to_string(),
                Value::String(".txt".to_string()),
            ),
            ("Name".to_string(), Value::String("c".to_string())),
        ]));

        let context = CmdletContext::with_input(vec![a, b, c])
            .with_parameter("Property".to_string(), Value::String("Extension".to_string()));

        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result.len(), 2);

        // Keys are sorted; expect ".rs" then ".txt".
        let names: Vec<String> = result
            .iter()
            .filter_map(|v| v.get_property("Name"))
            .map(|v| v.to_string())
            .collect();
        assert_eq!(names, vec![".rs".to_string(), ".txt".to_string()]);

        let counts: Vec<f64> = result
            .iter()
            .filter_map(|v| v.get_property("Count"))
            .filter_map(|v| v.to_number())
            .collect();
        assert_eq!(counts, vec![2.0, 1.0]);
    }

    #[test]
    fn test_group_object_multiple_properties() {
        let cmdlet = GroupObjectCmdlet;

        let x1 = Value::Object(HashMap::from([
            ("A".to_string(), Value::String("one".to_string())),
            ("B".to_string(), Value::String("two".to_string())),
        ]));
        let x2 = Value::Object(HashMap::from([
            ("A".to_string(), Value::String("one".to_string())),
            ("B".to_string(), Value::String("two".to_string())),
        ]));
        let y = Value::Object(HashMap::from([
            ("A".to_string(), Value::String("one".to_string())),
            ("B".to_string(), Value::String("three".to_string())),
        ]));

        let context = CmdletContext::with_input(vec![x1, x2, y]).with_parameter(
            "Property".to_string(),
            Value::Array(vec![Value::String("A".to_string()), Value::String("B".to_string())]),
        );

        let mut evaluator = pwsh_runtime::Evaluator::new();
        let result = cmdlet.execute(context, &mut evaluator).unwrap();
        assert_eq!(result.len(), 2);

        // Names are "one,three" and "one,two"; sorted lexicographically
        let names: Vec<String> = result
            .iter()
            .filter_map(|v| v.get_property("Name"))
            .map(|v| v.to_string())
            .collect();
        assert_eq!(names, vec!["one,three".to_string(), "one,two".to_string()]);
    }
}
