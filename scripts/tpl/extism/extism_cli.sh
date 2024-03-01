#!/bin/bash

# Simple base example to load and run a Dilla design system with WASM #TYPE# with Extism CLI.
# https://dilla.io

if ! command -v extism &>/dev/null; then
  echo -e "[Error] 'Extism CLI' could not be found, please install: https://extism.org/docs/install/"
fi

_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo -e "[Notice] Run 'describe components::_list'"
echo -e ""

extism call --wasi --input 'components::_list' "${_DIR}/#DS#_dev.wasm" describe

echo -e ""
echo -e "[Notice] Render index.json"
echo -e ""

_payload=$(<"${_DIR}/payload/index.json")
_result=$(extism call --wasi --input "${_payload}" "${_DIR}/#DS#.wasm" render)

# extism call --wasi --input '/tmp/payload/index.json' --allow-path .:/tmp ./#DS#_dev.wasm render

echo "${_result}" | python -m json.tool