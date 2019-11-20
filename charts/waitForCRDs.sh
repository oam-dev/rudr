#!/bin/bash

RESPONSE_STRING=''
while [[ $RESPONSE_STRING != "No resources found." ]]; do
  echo "Waiting for CRD persistence to finish..."
  RESPONSE_STRING=$((kubectl get trait) 2>&1 >/dev/null)
  sleep 10
done
echo "rudr CRDs have been persisted to the value cache."