package main

import (
	"flag"
	"fmt"
	"log"
	"net/http"
	"os"

	"k8s.io/client-go/kubernetes"
	"k8s.io/client-go/tools/clientcmd"

	"github.com/microsoft/scylla/pkg/controller"
)

const envScyllaHostAddress = "SCYLLA_HOST_ADDRESS"
const envNamespace = "KUBERNETES_NAMESPACE"

func main() {
	var (
		kubeconfig string
		master     string
	)

	flag.StringVar(&kubeconfig, "kubeconfig", "", "absolute path to the kubeconfig file")
	flag.StringVar(&master, "master", "", "master url for Kubernetes API server")

	// Start controller
	go startController(master, kubeconfig)
	// Start health check
	go startHealthz()
	select {}
}

func startController(master, kubeconfig string) {

	// creates the connection
	config, err := clientcmd.BuildConfigFromFlags(master, kubeconfig)
	if err != nil {
		log.Fatal(err)
	}

	// creates the clientset
	clientset, err := kubernetes.NewForConfig(config)
	if err != nil {
		log.Fatal(err)
	}

	ctrlConfig := controller.Config{
		Namespace:   envOr(envNamespace, "default"),
		Threadiness: 50,
	}

	ctrl := controller.New(clientset, ctrlConfig)
	ctrl.Run()
}

func startHealthz() {
	http.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprintf(w, "OK")
	})
	log.Fatal(http.ListenAndServe(envOr(envScyllaHostAddress, ":8080"), nil))
}

func envOr(varname string, defaultVal string) string {
	if val, ok := os.LookupEnv(varname); ok {
		return val
	}
	return defaultVal
}
