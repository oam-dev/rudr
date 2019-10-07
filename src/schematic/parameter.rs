use failure::Error;
use std::collections::BTreeMap;

pub type ParameterList = Vec<Parameter>;

/// Parameter describes a configurable unit on a Component or Application.
///
/// Parameters have primitive types, and may be marked as required. Default values
/// may be provided as well.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,
    pub description: Option<String>,

    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub parameter_type: ParameterType,

    #[serde(default = "default_required")]
    pub required: bool,

    pub default: Option<serde_json::Value>,
}

impl Parameter {
    fn validate(&self, val: &serde_json::Value) -> Result<(), Error> {
        match self.parameter_type {
            ParameterType::Boolean => val
                .as_bool()
                .ok_or_else(|| format_err!("expected boolean value for {}", self.name.as_str()))
                .and(Ok(())),
            ParameterType::String => val
                .as_str()
                .ok_or_else(|| format_err!("expected string value for {}", self.name.as_str()))
                .and(Ok(())),
            ParameterType::Number => {
                // AFAIK, there is no numeric value in JSON that cannot be represented as an f64.
                val.as_f64()
                    .ok_or_else(|| format_err!("expected numeric value for {}", self.name.as_str()))
                    .and(Ok(()))
            }
            ParameterType::Null => {
                // Not entirely clear what we want to do here.
                val.as_null()
                    .ok_or_else(|| format_err!("expected null value for {}", self.name.as_str()))
            }
        }
    }
}

pub type ResolvedVals = BTreeMap<String, serde_json::Value>;

pub fn resolve_value(
    params: ResolvedVals,
    from_param: Option<String>,
    value: Option<String>,
) -> Option<String> {
    match from_param {
        Some(p) => {
            params
                .get(p.as_str())
                .and_then(|i| {
                    // Not sure what to do for other types.
                    match i {
                        serde_json::Value::String(s) => Some(s.clone()),
                        _ => Some(i.to_string()),
                    }
                })
                .or_else(|| value.clone())
        }
        None => value.clone(),
    }
}

#[derive(Fail, Debug)]
#[fail(display = "validation failed: {:?}", errs)]
pub struct ValidationErrors {
    errs: Vec<Error>,
}

pub fn resolve_parameters(
    definition: Vec<Parameter>,
    values: ResolvedVals,
) -> Result<ResolvedVals, ValidationErrors> {
    let mut errors: Vec<Error> = Vec::new();
    let mut resolved: ResolvedVals = BTreeMap::new();

    definition
        .iter()
        .map(|d| {
            let resolved = match values.get(d.name.as_str()) {
                Some(val) => ParameterValue {
                    name: d.name.clone(),
                    value: Some(val.clone()),
                    from_param: None,
                },
                None => ParameterValue {
                    name: d.name.clone(),
                    value: d.default.clone().or(Some(serde_json::Value::Null)),
                    from_param: None,
                },
            };
            // Validation:
            if d.required
                && (resolved.value.is_none() || resolved.value.as_ref().unwrap().is_null())
            {
                errors.push(format_err!("parameter {} is required", d.name.clone()));
            }
            if let Err(e) = d.validate(resolved.value.as_ref().unwrap()) {
                errors.push(e)
            };
            resolved
        })
        .for_each(|p| {
            resolved.insert(p.name, p.value.unwrap());
        });
    if !errors.is_empty() {
        return Err(ValidationErrors { errs: errors });
    }
    Ok(resolved)
}

/// Resolve current values with material from parent values and return a map of name/value pairs.
///
/// If the current values have a `from` directive, the `from will be looked up in parent.
///
/// If a `from` fails to resolve, this will return
/// an error.
pub fn resolve_values(
    current: Vec<ParameterValue>,
    parent: Vec<ParameterValue>,
) -> Result<ResolvedVals, Error> {
    let mut merged: ResolvedVals = BTreeMap::new();

    for p in current.iter() {
        // If a `from_param` exists, get the value out of the parent. Otherwise, just use
        // this parameter's value.

        let new_val = p
            .from_param
            .clone()
            .and_then(|from_name| {
                let parent_override = parent.iter().find(|item| item.name == from_name);
                parent_override.and_then(|s| s.value.clone())
            })
            .or_else(|| p.value.clone());

        // If a from_param was not found, and no default value was supplied, then this is
        // an error.
        if p.from_param.is_some() && new_val.is_none() {
            return Err(format_err!(
                "could not resolve fromParam:{} for {}",
                p.from_param.clone().unwrap(),
                p.name.clone()
            ));
        }

        // If a parameter has neither a from nor a value, we just ignore it.
        if new_val.is_some() {
            merged.insert(p.name.clone(), new_val.unwrap());
        }
    }
    Ok(merged)
}

/// Supplies the default value for all required fields.
fn default_required() -> bool {
    false
}

/// ParameterType defines the types of parameters for a Parameters object.
///
/// These roughly correlate with JSON Schema primitive types.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ParameterType {
    Boolean,
    String,
    Number,
    Null,
}

/// A value that is substituted into a parameter.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ParameterValue {
    pub name: String,
    pub value: Option<serde_json::Value>,
    pub from_param: Option<String>,
}
