
#echo "Checking the rudr CRDs to see if they are ready..."
RESPONSE_STRING=''
while [[ $RESPONSE_STRING != "No resources found." ]]; do
  echo "Waiting for CRD caching to finish..."
  RESPONSE_STRING=$((kubectl get trait) 2>&1 >/dev/null)
  # echo "Hey: $RESPONSE_STRING"
  sleep 10
done
echo "CRDs have been cached..."
# echo $RESPONSE_STRING
