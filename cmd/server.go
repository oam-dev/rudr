package main

import (
	"fmt"
	"log"
	"net/http"
	"os"
)

func main() {
	// Start server
	go informer()
	// Start health check
	go healthz()

	select {}
}

func informer() {

}

func healthz() {
	http.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprintf(w, "OK")
	})
	log.Fatal(http.ListenAndServe(":8080", nil))
}

func envOr(varname string, defaultVal string) string {
	if val, ok := os.LookupEnv(varname); ok {
		return val
	}
	return defaultVal
}
