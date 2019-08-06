package controller

import "k8s.io/client-go/kubernetes"

// Config holds the configurable aspects of this controller.
type Config struct {
	Namespace   string
	Threadiness int
	StopChan    chan struct{}
}

// Controller is a Scylla Kubernetes controller
type Controller struct {
	clientset kubernetes.Interface
	config    Config
}

// New creates an initialized Controller.
func New(clientset kubernetes.Interface, config Config) *Controller {
	return &Controller{
		clientset: clientset,
		config:    config,
	}
}

// Run starts a controller.
func (c *Controller) Run() {

}
