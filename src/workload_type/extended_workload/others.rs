use async_trait::async_trait;
use crate::schematic::GroupVersionKind;
use crate::workload_type::{
    InstigatorResult, StatusResult, ValidationResult, WorkloadMetadata, WorkloadType,
};
use failure::{format_err, Error};
use kube::api::{PatchParams, PostParams, CustomResource};
use serde_json::json;
use std::collections::BTreeMap;

pub struct Others {
    pub meta: WorkloadMetadata,
    pub gvk: GroupVersionKind,
}

impl Others {
    pub fn new(meta: WorkloadMetadata, type_: &str) -> Result<Self, Error> {
        let gvk: GroupVersionKind = type_.parse()?;
        if meta
            .definition
            .workload_settings
            .iter()
            .find(|&item| item.name == "spec")
            .is_none()
        {
            return Err(format_err!(
                "unknown workload type must have spec in workloadSettings"
            ));
        };
        Ok(Others { meta, gvk })
    }
    pub fn get_object(&self) -> serde_json::Value {
        let api_version = self.gvk.group.clone() + "/" + self.gvk.version.as_str();
        let item = self
            .meta
            .definition
            .workload_settings
            .iter()
            .find(|&item| item.name == "spec")
            .unwrap();

        json!({
            "apiVersion": api_version,
            "kind": self.gvk.kind.clone(),
            "metadata": {
                "name": self.meta.instance_name.clone(),
                "ownerReferences": self.meta.owner_ref.clone(),
            },
            "spec": item.value,
        })
        //TODO now we only copy spec here, we could use json patch or something else to enable parameter override.
    }
}

fn form_plural(word: &str) -> String {
    if word.is_empty() {
        return word.to_string();
    }
    let mut newword = word.to_string();
    if newword.ends_with('y') {
        newword.pop();
        newword = newword.to_string() + "ies";
        return newword;
    }
    if newword.ends_with('s')
        || newword.ends_with('c')
        || newword.ends_with("ch")
        || newword.ends_with("sh")
    {
        return newword + "es";
    }
    newword + "s"
}

#[async_trait]
impl WorkloadType for Others {
    async fn add(&self) -> InstigatorResult {
        let crd_resource = CustomResource::kind(
            form_plural(self.gvk.kind.clone().to_lowercase().as_str()).as_str(),
        )
        .version(self.gvk.version.as_str())
        .group(self.gvk.group.as_str())
        .within(self.meta.namespace.as_str())
        .into_resource();
        let object = self.get_object();
        let crd_req = crd_resource.create(&PostParams::default(), serde_json::to_vec(&object)?)?;
        let _: serde_json::Value = self.meta.client.request(crd_req).await?;
        Ok(())
    }

    async fn modify(&self) -> InstigatorResult {
        let crd_resource = CustomResource::kind(
            form_plural(self.gvk.kind.clone().to_lowercase().as_str()).as_str(),
        )
        .version(self.gvk.version.as_str())
        .group(self.gvk.group.as_str())
        .within(self.meta.namespace.as_str())
        .into_resource();
        let object = self.get_object();
        let crd_req = crd_resource.patch(
            self.meta.instance_name.clone().as_str(),
            &PatchParams::default(),
            serde_json::to_vec(&object)?,
        )?;
        let _: serde_json::Value = self.meta.client.request(crd_req).await?;
        Ok(())
    }

    async fn delete(&self) -> InstigatorResult {
        Ok(())
    }

    async fn status(&self) -> StatusResult {
        // TODO: how to implement status while we don't know the spec?
        Ok(BTreeMap::new())
    }

    async fn validate(&self) -> ValidationResult {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::schematic::component::{Component, WorkloadSetting};
    use crate::schematic::parameter::ParameterType;
    use crate::workload_type::extended_workload::others::{form_plural, Others};
    use crate::workload_type::WorkloadMetadata;
    use kube::client::Client;
    use kube::config::Config;
    use serde_json::json;
    use std::collections::BTreeMap;

    #[test]
    fn test_get_object() {
        let workload = Others::new(
            WorkloadMetadata {
                name: "test".to_string(),
                component_name: "test".to_string(),
                instance_name: "test".to_string(),
                namespace: "default".to_string(),
                definition: Component {
                    workload_settings: vec![WorkloadSetting {
                        name: "spec".to_string(),
                        parameter_type: ParameterType::Object,
                        value: Some(
                            serde_json::to_value(json!({"image":"testrepo/test","name":"test"}))
                                .unwrap(),
                        ),
                        from_param: None,
                        required: true,
                        description: None,
                    }],
                    ..Default::default()
                },
                client: Client::new(Config::new(
                    ".".parse().unwrap(),
                )),
                params: BTreeMap::new(),
                owner_ref: None,
                annotations: None,
            },
            "extend.oam.dev/v1alpha1.Test",
        )
        .unwrap();

        assert_eq!(
            json!({"apiVersion":"extend.oam.dev/v1alpha1","kind":"Test","metadata":{"name":"test","ownerReferences":null},"spec":{"image":"testrepo/test","name":"test"}}),
            workload.get_object()
        )
    }

    #[test]
    fn test_form_plural() {
        assert_eq!("functions", form_plural("function").as_str());
        assert_eq!("prometheuses", form_plural("prometheus").as_str());
    }
}
