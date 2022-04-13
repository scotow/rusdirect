#!/usr/bin/env bash

set -e -u -o pipefail

readonly DOMAIN=""

if [ -z "$DOMAIN" ]; then
  echo "Unspecified domain" >&2
  exit 1
fi

if [ $# -ge 1 ]; then
  declare SHORT="$1"
else
  declare SHORT=""
fi

if [ $# -ge 2 ]; then
  declare TARGET="$2"
else
  declare TARGET="$(cat)"
fi

curl -s -X POST --data-urlencode path="$SHORT" --data-urlencode target="$TARGET" "$DOMAIN"