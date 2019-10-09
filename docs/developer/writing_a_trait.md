# Writing a Trait

This guide explains how to write a trait for Rudr, using the VolumeMounter trait as an example.

> Note: The process for adding a new trait is currently more difficult than it needs to be. In future versions of Rudr, we will be streamlining this process considerably.

## Step 1: Defining the Trait Resource

The first thing we will do is define a trait resource as a YAML file that Helm will install.

```yaml
---
apiVersion: core.oam.dev/v1alpha1
kind: Trait
metadata:
  # Define the name of the trait. This will be the reference by which ApplicationConfigurations
  # reference this trait.
  name: volume-mounter
spec:
  # The appliesTo field lists all of the Workload Types that this trait can be added to.
  # In this case, all of the core workload types are supported.
  appliesTo:
    - core.oam.dev/v1alpha1.Server
    - core.oam.dev/v1alpha1.SingletonServer
    - core.oam.dev/v1alpha1.Worker
    - core.oam.dev/v1alpha1.SingletonWorker
    - core.oam.dev/v1alpha1.Task
    - core.oam.dev/v1alpha1.SingletonTask
  # Properties define what things can be configured on this trait.
  #
  # A property definition requires four fields:
  # - name: The name of this property. AppConfig authors will use this to specify configuration
  # - description: User-friendly text describing the property
  # - type: The data type of this field. Typically, this is string, number, or boolean.
  # - required: If this is true, an AppConfiguration MUST provide a value for this property
  properties:
    - name: volumeName
      description: The name of the volume this backs. This matches the volume name declared in the ComponentSchematic.
      type: string
      required: true
    - name: storageClass
      description: The storage class that a PVC requires
      type: string
      required: true
```

The YAML above belongs at the bottom of `charts/rudr/templates/traits.yaml`.

## Step 2: Writing a Trait Struct

Now we're ready for some Rust code. Inside of the `src/schematic/traits` directory, add a new module.

In our case, we're adding `volume_mounter.rs`. Right now, we'll just stub out the main struct:

```rust
use crate::schematic::traits::util::OwnerRefs;

/// The VolumeMounter trait provisions volumes that can
/// be mounted by a Component.
pub struct VolumeMounter {
    /// The app configuration name
    pub name: String,
    /// The instance name for this component
    pub instance_name: String,
    /// The component name
    pub component_name: String,
    /// The owner reference (usually of the component instance).
    /// This should be attached to any Kubernetes resources that this trait creates.
    pub owner_ref: OwnerRefs,
    /// The component that we are attaching to
    pub component: Component,
    /// The name 
    pub volume_name: String,
    /// The name of the storage class to which this will derive a PVC
    pub storage_class: String,
}
```

Note that the first four items (`name`, `instance_name`, `component_name`, and `owner_ref`) are typically used by all traits. (At some point they will probably be refactored into their own object).

The last two fields, `volume_name` and `storage_class` are the two parameters we defined in our trait YAML above.

## Step 3: Declaring the New Trait Module
As soon as you have created the struct above, edit `src/traits.rs` and add a module declaration:

```rust
// Existing traits
// Re-exports
mod autoscaler;
pub use crate::schematic::traits::autoscaler::Autoscaler;
mod ingress;
pub use crate::schematic::traits::ingress::Ingress;
mod empty;
pub use crate::schematic::traits::empty::Empty;
mod manual_scaler;
pub use crate::schematic::traits::manual_scaler::ManualScaler;
// Our new trait
mod volume_mounter;
pub use crate::schematic::traits::volume_mounter::VolumeMounter;
pub const VOLUME_MOUNTER: &str = "volume-mounter";

// Then more code...

// Then a little further down:
pub enum OAMTrait {
    Autoscaler(Autoscaler),
    ManualScaler(ManualScaler),
    Ingress(Ingress),
    VolumeMounter(VolumeMounter), // <-- add this
    Empty(Empty),
}
impl OAMTrait {
    pub fn exec(&self, ns: &str, client: APIClient, phase: Phase) -> TraitResult {
        match self {
            OAMTrait::Autoscaler(a) => a.exec(ns, client, phase),
            OAMTrait::Ingress(i) => i.exec(ns, client, phase),
            OAMTrait::ManualScaler(m) => m.exec(ns, client, phase),
            OAMTrait::VolumeMounter(m) => m.exec(ns, client, phase), // <-- add this
            OAMTrait::Empty(e) => e.exec(ns, client, phase),
        }
    }
}
```

Save these files, and you should be able to compile now (even though nothing new will happen yet).

## Constructing a New Trait Instance

The next thing we need to do is write a constructor function for our new trait. This function will
take several arguments and create a new instance of our trait. While most trait constructors receive
the same arguments, this is not a requirement of the system.

Note that a constructor function is not responsible for validation of the data types or the `required` field. That is handled separately. The constructor function merely needs to create a workable instance.

```rust
impl VolumeMounter {
    pub fn from_params(
        name: String,
        instance_name: String,
        component_name: String,
        params: ParamMap,
        owner_ref: OwnerRefs,
        component: Component,
    ) -> Self {
        VolumeMounter {
            name,
            component_name,
            instance_name,
            owner_ref,
            component,
            volume_name: params
                .get("volumeName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            storage_class: params
                .get("storageClass")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }
    }
}
```

The most important part above is how `volume_name` and `storage_class` are both set by searching through the supplied params and converting the results to the expected type.

> Tip: If you want to see the imports, you may want to look at the code in [volume_mounter.rs](https://github.com/microsoft/scylla/blob/master/src/schematic/traits/volume_mounter.rs).

We can write a quick test at the bottom of `volume_mounter.rs` to verify that the parameters are correctly fetched:

```rust
#[cfg(test)]
mod test {
    use super::VolumeMounter;
    use crate::workload_type::ParamMap;
    #[test]
    fn test_from_params() {
        let mut params = ParamMap::new();
        params.insert("storageClass".into(), serde_json::json!("really-fast"));
        params.insert("volumeName".into(), serde_json::json!("panda-bears"));
        let vm = VolumeMounter::from_params(
            "name".to_string(),
            "instance name".to_string(),
            "component name".to_string(),
            params,
            None,
        );

        assert_eq!("really-fast", vm.storage_class);
        assert_eq!("panda-bears", vm.volume_name);
    }
}
```

## Defining a PersistentVolumeClaim

Next, we'll add a method to our trait to create a PVC. This is the thing that will actually provision our storage.

```rust
impl VolumeMounter {
    //... Stuff we did above

    /// Create a PersistentVolumeClaim that describes this volume
    pub fn to_pvc(&self) -> core::PersistentVolumeClaim {
        let mut reqs = BTreeMap::new();
        reqs.insert("storage".to_string(), Quantity("200M".to_string()));
        core::PersistentVolumeClaim {
            metadata: Some(meta::ObjectMeta {
                name: Some(self.volume_name.clone()),
                labels: Some(self.labels()), // self.labels() just generates a BTreeMap of labels.
                owner_references: self.owner_ref.clone(),
                ..Default::default()
            }),
            spec: Some(core::PersistentVolumeClaimSpec {
                access_modes: Some(vec!["ReadWriteOnce".to_string()]),
                storage_class_name: Some(self.storage_class.clone()),
                resources: Some(core::ResourceRequirements {
                    requests: Some(reqs),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
```

Above, we have written a function that creates a new PVC based on the `VolumeMounter`. The version above is very simple, not accounting for things like setting limits on size or requiring a read-only volume. But for our purposes here, it is sufficient.

Note that we use the `..Default::default()` pattern as frequently as possible rather than specifying the exact fields. We do this because it is easier to maintain in the future. As Kubernetes adds new optional fields, it will not require us to rewrite the code.

## Writing a TraitImplementation for the Trait

The last part of writing a trait is implementing the `TraitImplementation`. This is a straightforward process of describing how the trait deals with install, upgrade, and delete requests.

> Why is it called `TraitImplementation` instead of just `Trait`? Because Rust uses `Trait` as a reserved word. Though the original versions of Open Application Model were not written in Rust, the Open Application Model trait system was inspired by Rust traits.

In this part, we need to write an `impl TraitImplementation for VolumeMounter`:

```rust
impl TraitImplementation for VolumeMounter {
    fn add(&self, ns: &str, client: APIClient) -> TraitResult {
        let pvc = self.to_pvc();
        let (req, _) = core::PersistentVolumeClaim::create_namespaced_persistent_volume_claim(
            ns,
            &pvc,
            Default::default(),
        )?;
        client.request::<core::PersistentVolumeClaim>(req)?;
        Ok(())
    }
    fn modify(&self, ns: &str, client: APIClient) -> TraitResult {
        let pvc = self.to_pvc();
        let values = serde_json::to_value(&pvc)?;
        let (req, _) = core::PersistentVolumeClaim::patch_namespaced_persistent_volume_claim(
            self.volume_name.as_str(),
            ns,
            &meta::Patch::StrategicMerge(values),
            Default::default(),
        )?;
        client.request::<core::PersistentVolumeClaim>(req)?;
        Ok(())
    }
    fn delete(&self, ns: &str, client: APIClient) -> TraitResult {
        let (req, _) = core::PersistentVolumeClaim::delete_namespaced_persistent_volume_claim(
            self.volume_name.as_str(),
            ns,
            Default::default(),
        )?;
        client.request::<core::PersistentVolumeClaim>(req)?;
        Ok(())
    }
}
```

In a nutshell, all we are doing above is declaring how to add, modify, and remove a VolumeMounter. For our implementation, this is simply creating, patching, and deleting PVCs. Note: Next time choose an example whose name is shorter than `PersistenVolumeClaim`.

## Registering the Trait with the Trait Manager

Now that we have our trait written, the last step is to register it with the trait manager. This process tells Rudr to handle requests for our new trait.

The `src/instigator.rs` file has the trait manager. Find `impl TraitManager` and edit the `load_trait` function:

```rust
impl TraitManager {
    // ... ignore some stuff
    fn load_trait(&self, binding: &TraitBinding) -> Result<OAMTrait, Error> {

    }
}

```