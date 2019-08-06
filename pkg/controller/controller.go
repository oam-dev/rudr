package controller

import (
	"k8s.io/client-go/kubernetes"
	"k8s.io/client-go/tools/cache"
	"k8s.io/client-go/util/workqueue"
)

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
	queue     workqueue.RateLimitingInterface
	indexer   cache.Indexer
	informer  cache.Controller
}

// New creates an initialized Controller.
func New(clientset kubernetes.Interface, config Config) *Controller {
	c := &Controller{
		clientset: clientset,
		config:    config,
		queue:     workqueue.NewRateLimitingQueue(workqueue.DefaultControllerRateLimiter()),
	}
	return c.initInformer()
}

// Run starts a controller.
func (c *Controller) Run() {

}

func (c *Controller) initInformer() *Controller {
	// This informer watches for OperationalConfiguration objects.
	/*
		c.indexer, c.informer = cache.NewIndexerInformer(
			&cache.ListWatch{
				ListFunc: func(opts meta.ListOptions) (runtime.Object, error) {
					//return c.clientset.
				},
			},
			&schematic.OperationalConfiguration,
			0,
			cache.ResourceEventHandlerFuncs{},
			cache.Indexers{},
		)*/
	return c
}
