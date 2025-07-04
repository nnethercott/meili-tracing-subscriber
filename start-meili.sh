#!/bin/bash
set -xe

if [[ ! -x $(command -v docker) ]]; then
  echo "make sure docker is installed ! "
  exit 1
fi

MEILISEARCH_IMAGE="getmeili/meilisearch"

docker run \
  -p 7700:7700 \
  -d --rm "$MEILISEARCH_IMAGE"
  # -e MEILI_MASTER_KEY=password \
