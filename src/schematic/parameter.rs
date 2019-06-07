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
    fn validate(&self, val: &serde_json::Value) -> Result<(), failure::Error> {
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

    /// Get the value for this parameter.
    ///
    /// If the Option is empty, this will attempt to find a default value in the Parameter
    /// definition. Otherwise, it will validate the given value against the definition.
    ///
    /// Any successful value will be returned. Any failure (validation or no found value) will result in an error.
    fn get_value(&self, val: Option<&ParameterValue>) -> Result<serde_json::Value, failure::Error> {
        match val {
            Some(pval) => {
                let value = pval.value.clone();
                self.validate(&value).and(Ok(value))
            }
            None => {
                if self.required {
                    return Err(format_err!("value for {} is required", self.name.as_str()));
                }
                self.default
                    .clone()
                    .ok_or(format_err!("No value specified for {}", self.name.as_str()))
            }
        }
    }
}

type ResolvedVals = BTreeMap<String, serde_json::Value>;

#[derive(Fail, Debug)]
#[fail(display = "validation failed: {:?}", errs)]
pub struct ValidationErrors {
    errs: Vec<failure::Error>,
}

pub fn resolve_params(
    definition: Vec<Parameter>,
    values: Vec<ParameterValue>,
) -> Result<ResolvedVals, ValidationErrors> {
    let mut errors: Vec<failure::Error> = Vec::new();
    let mut resolved: ResolvedVals = BTreeMap::new();

    definition
        .iter()
        .map(|d| {
            let val = values.iter().find(|e| e.name == d.name);
            match d.get_value(val) {
                Ok(resolved_val) => ParameterValue {
                    name: d.name.clone(),
                    value: resolved_val,
                },
                Err(e) => {
                    errors.push(e);
                    ParameterValue {
                        name: d.name.clone(),
                        value: serde_json::Value::Null,
                    }
                }
            }
        })
        .for_each(|p| {
            resolved.insert(p.name, p.value);
            ()
        });
    if errors.len() > 0 {
        return Err(ValidationErrors { errs: errors });
    }
    Ok(resolved)
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
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParameterValue {
    name: String,
    value: serde_json::Value,
}
