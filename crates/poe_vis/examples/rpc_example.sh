#!/usr/bin/env bash

URL="http://0.0.0.0:6004"
PATH1=(10364 42857 20024 44223 49220 36778 36479 12925 61196 58329)
PATH2=(10364 42857 20024 44223 49220 14725 34233 32545 61196 58329)
PATH3=(10364 55342 17248 53960 8975 61196 58329)

activate() {
  curl -s -X POST -H "Content-Type: application/json" \
    --data "{\"jsonrpc\":\"2.0\",\"method\":\"activate_node\",\"params\":[$1],\"id\":1}" \
    "$URL"
  sleep 0.050
}

for node in "${PATH1[@]}"; do activate "$node"; done
for node in "${PATH2[@]}"; do activate "$node"; done
for node in "${PATH3[@]}"; do activate "$node"; done
