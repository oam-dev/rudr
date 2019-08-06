package controller

import (
	"testing"

	"github.com/stretchr/testify/assert"

	"k8s.io/client-go/kubernetes"
	"k8s.io/client-go/kubernetes/fake"
)

func TestController(t *testing.T) {
	is := assert.New(t)

	stopChan := make(chan struct{})
	defer close(stopChan)

	cfg := Config{
		Namespace:   "test",
		Threadiness: 1,
		StopChan:    stopChan,
	}
	cli := fakeClient()

	ctrl := New(cli, cfg)

	is.Equal("test", ctrl.config.Namespace)
	is.Equal(1, ctrl.config.Threadiness)
}

func fakeClient() kubernetes.Interface {
	return fake.NewSimpleClientset()
}
