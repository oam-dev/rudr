use super::parameter::ParameterValue;
use std::collections::BTreeMap;
use std::cmp::Ordering;
use failure::Error;
use regex::Regex;

fn parse_from_variable(input: String) -> Option<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^\[fromVariable\((?P<var>[[:word:]]+)\)\]$"#).unwrap();
    }
    RE.captures(&input)
        .and_then(|cap| cap.name("var").map(|var| var.as_str().to_owned()))
}

/// Variables are common values that can be substituted into
/// predefined locations within an application configuration
/// using the [fromVariable(VARNAME)] syntax.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Variable {
    /// The variable's name (must be unique per configuration).
    pub name: String,
    /// The variable's name scalar value.
    pub value: serde_json::Value,
}

impl From<Variable> for ParameterValue {
    fn from(var: Variable) -> Self {
        ParameterValue {
            name: var.name.clone(),
            value: Some(var.value.clone()),
            from_param: None,
        }
    }
}

impl PartialOrd for Variable {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Ord for Variable {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl Eq for Variable {}

/// Expand any variables in current values using the variables defined in the configuration.
/// 
/// If a variable is referenced but undefined, the parameter value will be set to None.
pub fn expand_variables(values: &mut Vec<ParameterValue>, vars: BTreeMap<String, serde_json::Value>) -> Result<(), Error> {
    for param in values.iter_mut() {
        if let Some(value) = &param.value {
            if let serde_json::Value::String(s) = value {
                if let Some(ref var) = parse_from_variable(s.clone()) {
                    param.value = vars.get(var).map(|var| var.clone());
                    ensure!(param.value.is_some(), format!(
                        "parameter `{:?}` references undefined variable `{:?}`",
                        &param.name,
                        &var,
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Resolve parameter values containing variables.
pub fn resolve_variables(values: Vec<ParameterValue>, vars: Vec<Variable>) -> Result<Vec<ParameterValue>, Error> {
    expand_variables(
        &mut values.clone(),
        vars
            .into_iter()
            .map(|var| (var.name.clone(), var.value.clone()))
            .collect::<BTreeMap<String, serde_json::Value>>())?;
    Ok(values.to_vec())

}

/// Transform a vector of variables into parameter values.
pub fn get_variable_values(vars: Option<Vec<Variable>>) -> Vec<ParameterValue> {
    let mut vars = vars.unwrap_or(vec![]);
    dedup(&mut vars);
    vars.into_iter()
        .map(|var| var.into())
        .collect()
}

// TODO: variables are unique per config should redefinition should be an error.
pub fn dedup<T: Ord>(values: &mut Vec<T>) {
    values.sort_unstable();
    values.dedup();
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use super::*;

    #[test]
    fn test_resolve_variables() {
        resolve_variables(
            vec![
                ParameterValue {
                    name: "dinner1".into(),
                    value: Some(json!("[fromVariable(pet1)]")),
                    from_param: None,
                },
                ParameterValue {
                    name: "dinner2".into(),
                    value: Some(json!("[fromVariable(pet2)]")),
                    from_param: None,
                },
            ],
            vec![
                Variable{name: "pet1".into(), value: json!("cat")},
                Variable{name: "pet2".into(), value: json!("dog")},
            ],
        )
        .expect("resolve variables");

        // test parameter value referencing undefine variable should error.
        resolve_variables(
            vec![
                ParameterValue {
                    name: "dinner".into(),
                    value: Some(json!("[fromVariable(cereal)]")),
                    from_param: None,
                },
            ],
            vec![],
        ).expect_err(r#"undefined variable `"cereal"`"#);
    }

    #[test]
    fn test_parse_from_variable() {
        assert_eq!(
            Some("VAR_42".to_owned()),
            parse_from_variable("[fromVariable(VAR_42)]".into())
        );
        assert_eq!(
            Some("_".to_owned()),
            parse_from_variable("[fromVariable(_)]".into())
        );
        assert_eq!(
            Some("42".to_owned()),
            parse_from_variable("[fromVariable(42)]".into())
        );
        assert_eq!(
            Some("VAR".to_owned()),
            parse_from_variable("[fromVariable(VAR)]".into())
        );
        assert_eq!(None, parse_from_variable("[fromVariable (VAR)]".into())); // illegal
        assert_eq!(None, parse_from_variable("[fromVariable()]".into()));
    }
}