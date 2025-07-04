#!/bin/bash

# Append a pr number to the worker name
sed -i "s/\"mattcraig-tech\"/\"mattcraig-tech-pr${PR_NUMBER}\"/" wrangler.toml
# Test for an existing kv in this worker's namespace bound to STATIC and create one if not present
npx wrangler kv namespace list > namespaces.txt
if grep -q mattcraig-tech-pr${PR_NUMBER}-STATIC namespaces.txt; then
	NAMESPACE="{ binding = \"STATIC\", id = $(cat namespaces.txt | sed ":again;\$!N;\$!b again; s/.*\({[^}]*mattcraig-tech-pr${PR_NUMBER}-STATIC[^}]*}\).*/\1/" | sed ':again;$!N;$!b again; s/.*\"id\": \([^,]*\).*/\1/') }"
else
	NAMESPACE=$(npx wrangler kv namespace create STATIC | sed ':again;$!N;$!b again; s/.*\({[^}]*}\).*/\1/g')
fi
echo $NAMESPACE
sed -i "s/{[^}]*}/${NAMESPACE}/g" wrangler.toml
cat wrangler.toml
