#!/usr/bin/env bash

set -euo pipefail

[[ "${VERBOSE:-}" == "1" ]] && set -x

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OPENAPI_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
PRETTY_OUTPUT="${OPENAPI_DIR}/generated/openapi.pretty.json"
OUTPUT="${OPENAPI_DIR}/generated/openapi.json"

cleanup() {
    rm -f "${PRETTY_OUTPUT}"
}

check_dependency() {
    local dependency="$1"

    if ! command -v "${dependency}" >/dev/null 2>&1; then
        echo "error: '${dependency}' is required" >&2
        echo "please install '${dependency}' and try again" >&2
        exit 1
    fi
}

bundle_openapi() {
    mkdir -p "$(dirname "${OUTPUT}")"

    docker run --rm \
        -v "${OPENAPI_DIR}:/workspace" \
        -w /workspace \
        redocly/cli \
        bundle spec/openapi.yaml \
        -o generated/openapi.pretty.json

    jq -c . "${PRETTY_OUTPUT}" > "${OUTPUT}"
    echo "OpenAPI bundle generated: ${OUTPUT}"
}

main() {
    check_dependency docker
    check_dependency jq

    bundle_openapi
}

trap cleanup EXIT
main

