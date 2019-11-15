use failure::Error;
use k8s_openapi::api::core::v1::ObjectReference;
use kube::{api::Api, api::PostParams, client::APIClient};
use std::fmt;

pub enum Type {
    Normal,
    Warning,
}
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Normal => write!(f, "Normal"),
            Type::Warning => write!(f, "Warning"),
        }
    }
}

#[derive(Clone)]
pub struct Event {
    pub client: APIClient,
    pub namespace: String,
    pub reporting_component: Option<String>, // Name of the controller that emitted this Event,e.g. "oam.dev/rudr"
    pub reporting_instance: Option<String>,  //ID of the controller instance
    pub event_handle: Api<kube::api::v1Event>,
}

pub struct Info {
    pub action: String,
    pub message: String,
    pub reason: String,
}

impl Event {
    pub fn new(client: APIClient, namespace: String) -> Self {
        Event {
            client: client.clone(),
            namespace: namespace.clone(),
            reporting_component: None,
            reporting_instance: None,
            event_handle: Api::v1Event(client).within(namespace.as_str()),
        }
    }
    fn make_event(
        now: chrono::DateTime<chrono::Utc>,
        namespace: String,
        type_: Type,
        info: Info,
        involved_object: ObjectReference,
        reporting_component: Option<String>,
        reporting_instance: Option<String>,
    ) -> kube::api::v1Event {
        let name = format!(
            "{}.{:x}",
            involved_object.name.clone().unwrap(),
            now.timestamp_nanos(),
        );
        kube::api::v1Event {
            metadata: kube::api::ObjectMeta {
                name,
                namespace: Some(namespace.clone()),
                ..Default::default()
            },
            involvedObject: involved_object,
            reportingComponent: reporting_component.unwrap_or_else(|| "".to_string()),
            reportingInstance: reporting_instance.unwrap_or_else(|| "".to_string()),
            message: info.message,
            reason: info.reason,
            count: 1,
            type_: type_.to_string(),
            action: Some(info.action),
            eventTime: None,
            firstTimestamp: Some(k8s_openapi::apimachinery::pkg::apis::meta::v1::Time(
                now.clone(),
            )),
            lastTimestamp: Some(k8s_openapi::apimachinery::pkg::apis::meta::v1::Time(
                now.clone(),
            )),
            related: None,
            series: None,
            source: None,
        }
    }
    pub fn push_event_message(
        &self,
        type_: Type,
        info: Info,
        involved_object: ObjectReference,
    ) -> Result<(), Error> {
        let now = chrono::Utc::now();
        let event = Event::make_event(
            now,
            self.namespace.clone(),
            type_,
            info,
            involved_object,
            self.reporting_component.clone(),
            self.reporting_instance.clone(),
        );
        self.event_handle
            .create(&PostParams::default(), serde_json::to_vec(&event)?)?;
        Ok(())
    }
}

#[test]
fn test_make_event() {
    let now = chrono::Utc::now();
    let ev = Event::make_event(
        now,
        "default".to_string(),
        Type::Normal,
        Info {
            action: "ac".to_string(),
            message: "ms".to_string(),
            reason: "re".to_string(),
        },
        ObjectReference {
            api_version: Some("core.oam.dev/v1alpha1".to_string()),
            kind: None,
            name: Some("test".to_string()),
            field_path: None,
            namespace: None,
            resource_version: None,
            uid: None,
        },
        None,
        None,
    );
    assert_eq!(ev.count, 1);
    assert_eq!(
        ev.metadata.name,
        format!("test.{:x}", now.timestamp_nanos())
    );
    assert_eq!(ev.action, Some("ac".to_string()))
}
