#!/bin/bash

# Simple base example to load and run a Dilla design system with WASM #TYPE# with Wasmtime.
# https://dilla.io

_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${_DIR}" || exit

_result=$(wasmtime --dir=. #DS#.core.wasm render payload/index.json)

echo "${_result}" | python -m json.tool
