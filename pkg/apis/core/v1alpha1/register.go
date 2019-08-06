package v1alpha1

import (
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/runtime"
	"k8s.io/apimachinery/pkg/runtime/schema"
)

// SchemeGroupVersion defines the group and version for this package.
var SchemeGroupVersion = schema.GroupVersion{
	Group:   "core.hydra.io",
	Version: "v1alpha1",
}

// Resource is magic. It gets called by the generated code.
func Resource(res string) schema.GroupResource {
	return SchemeGroupVersion.WithResource(res).GroupResource()
}

var (
	// SchemeBuilder is a singleton used by the generated code.
	SchemeBuilder      runtime.SchemeBuilder
	localSchemeBuilder = &SchemeBuilder
	// AddToScheme wraps the tools to add these resources to the scheme.
	AddToScheme = localSchemeBuilder.AddToScheme
)

func init() {
	localSchemeBuilder.Register(addKnownTypes)
}

func addKnownTypes(scheme *runtime.Scheme) error {
	scheme.AddKnownTypes(
		SchemeGroupVersion,
		&OperationalConfiguration{},
		&OperationalConfigurationList{},
	)

	scheme.AddKnownTypes(
		SchemeGroupVersion,
		&meta.Status{},
	)

	meta.AddToGroupVersion(scheme, SchemeGroupVersion)

	return nil
}
