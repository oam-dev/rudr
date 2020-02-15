use failure::Error;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::client::APIClient;
use log::{debug, error};
use serde_json::json;
use serde_json::map::Map;
use std::collections::BTreeMap;

use crate::{
    lifecycle::Phase,
    schematic::{
        component::Component,
        configuration::ComponentConfiguration,
        parameter::ParameterValue,
        traits::{
            self, Autoscaler, Empty, Ingress, ManualScaler, OAMTrait, TraitBinding, VolumeMounter,
        },
    },
};

// TraitManager maps a component to its traits, and handles trait lifecycle.
//
// Each component configuration is assigned a trait manager. That trait manager
// can load all of the associated traits, and then executed phases for each of
// the traits.
pub struct TraitManager {
    pub config_name: String,
    pub instance_name: String,
    pub component: ComponentConfiguration,
    pub parent_params: Vec<ParameterValue>,
    pub owner_ref: Option<Vec<meta::OwnerReference>>,
    pub workload_type: String,
    // Component schematic loaded from cluster.
    pub component_schematic: Component,
    pub traits: Vec<OAMTrait>,
}

impl TraitManager {
    pub fn load_traits(&mut self) -> Result<(), failure::Error> {
        let mut traits: Vec<OAMTrait> = vec![];
        for t in self.component.traits.as_ref().unwrap_or(&vec![]).iter() {
            // Load all of the traits into the manager.
            let imp = self.load_trait(&t)?;
            traits.push(imp);
        }
        self.traits = traits;
        Ok(())
    }
    fn load_trait(&self, binding: &TraitBinding) -> Result<OAMTrait, failure::Error> {
        debug!("Trait binding params: {:?}", &binding.parameter_values);
        let empty_value_ref: &serde_json::Value = &json!("");
        let prop_map: Option<&Map<String, serde_json::value::Value>> = binding
            .properties
            .as_ref()
            .unwrap_or_else(|| empty_value_ref)
            .as_object();
        match binding.name.as_str() {
            traits::INGRESS_V1ALPHA1 => {
                let ing = Ingress::from_properties(
                    self.config_name.clone(),
                    self.instance_name.clone(),
                    self.component.component_name.clone(),
                    prop_map,
                    self.owner_ref.clone(),
                );
                debug!("INGRESS_V1ALPHA1: {:?}", ing);
                Ok(OAMTrait::Ingress(ing))
            }
            traits::VOLUME_MOUNTER_V1ALPHA1 => {
                let volmount = VolumeMounter::from_properties(
                    self.config_name.clone(),
                    self.instance_name.clone(),
                    self.component.component_name.clone(),
                    prop_map,
                    self.owner_ref.clone(),
                    self.component_schematic.clone(),
                );
                debug!("VOLUME_MOUNTER: {:?}", volmount);
                Ok(OAMTrait::VolumeMounter(Box::new(volmount)))
            }
            traits::AUTOSCALER_V1ALPHA1 => {
                let auto_scaler = Autoscaler::from_properties(
                    self.config_name.clone(),
                    self.instance_name.clone(),
                    self.component.component_name.clone(),
                    prop_map,
                    self.owner_ref.clone(),
                );
                debug!("Auto_scaler: {:?}", auto_scaler);
                Ok(OAMTrait::Autoscaler(auto_scaler))
            }
            traits::MANUAL_SCALER_V1ALPHA1 => {
                let scaler = ManualScaler::from_properties(
                    self.config_name.clone(),
                    self.instance_name.clone(),
                    self.component.component_name.clone(),
                    prop_map,
                    self.owner_ref.clone(),
                    self.workload_type.clone(),
                );
                debug!("Manual_scaler: {:?}", scaler);
                Ok(OAMTrait::ManualScaler(scaler))
            }
            // Empty is a debugging tool for checking whether the traits system is functioning independently of
            // its environment.
            traits::EMPTY => {
                let empty = Empty {};
                Ok(OAMTrait::Empty(empty))
            }
            _ => Err(format_err!("unknown trait {}", binding.name)),
        }
    }
    pub async fn exec(&self, ns: &str, client: APIClient, phase: Phase) -> Result<(), Error> {
        for imp in &self.traits {
            // At the moment, we don't return an error if a trait fails.
            let res = imp.exec(ns, client.clone(), phase.clone()).await;
            if let Err(err) = res {
                error!(
                    "Trait phase {:?} failed for {}: {:?}",
                    phase,
                    self.config_name.as_str(),
                    err
                );
            }
        }
        Ok(())
    }
    pub async fn status(&self, ns: &str, client: APIClient) -> Option<BTreeMap<String, String>> {
        let mut all_status = BTreeMap::new();
        for imp in &self.traits {
            if let Some(status) = imp.status(ns, client.clone()).await {
                for (name, state) in status {
                    //we don't need to worry about name conflict as K8s wouldn't allow this happen in the same namespace.
                    all_status.insert(name, state);
                }
            };
        }
        if all_status.is_empty() {
            return None;
        }
        Some(all_status)
    }
}
