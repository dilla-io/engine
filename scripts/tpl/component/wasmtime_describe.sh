#!/bin/bash

# Simple base example to load and run a Dilla design system with WASM #TYPE# with Wasmtime.
# https://dilla.io

_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

wasmtime run "${_DIR}/#DS#_dev.core.wasm" describe components::_list
