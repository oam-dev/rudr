
echo "Checking the scylla CRDs to see if they are ready..."
RESPONSE_STRING=''
while [[ $RESPONSE_STRING != "No resources found." ]]; do
  echo "Waiting for CRD caching to finish..."
  RESPONSE_STRING=$(kubectl get trait)
  echo "Hey: $RESPONSE_STRING"
  sleep 10
done
echo $RESPONSE_STRING
