#!/bin/bash
#
# Dilla WASM Extism builder from Design System.
# shellcheck disable=SC2086

_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# shellcheck source=/dev/null
if [ -f "${_DIR}/_init.sh" ]; then source "${_DIR}/_init.sh"; else
  echo -e "\e[31m[Error] Missing _init.sh file!\e[0m"
  exit 1
fi

_print_help() {
  cat <<HEREDOC
Dilla WASM Extism builder from Design System.

Argument:
  Dilla Design System name

Commands:
  run              Build a single Design System WASM Extism
  all              Build all Design System WASM Extism
  info | i         Get variables info used by this script

Options:
  -v --verbose     Run script with more output
  -h --help        Display this help information

Usage:
  ${_ME} run [_DS_NAME_]
  ${_ME} all
HEREDOC
}

###############################################################################
# Specific script variables
###############################################################################

__generated_wasm="${_DILLA_ROOT_DIR}/target/wasm32-wasi/release/wasm_extism.wasm"

###############################################################################
# Script functions
###############################################################################

_run() {
  _init
  _valid_ds
  __build_wasm
}

_all() {
  IFS=, read -ra values <<< "$DILLA_DS_NAMES"
  for _ds in "${values[@]}"; do DS="${_ds}" _run; done
}

_docker_run() {
  _docker_run_build "ex"
}

_docker_all() {
  _docker_build_all "ex"
}

###############################################################################
# Programs internal functions
###############################################################################

__build_wasm() {
  _log_info "Generate WASM Extism for: ${DS} v${VERSION}..."
  if ! ((_NO_BUILD)); then
    __prepare_dir "${_DILLA_DS_TARGET}"
    __build_wasm_extism
  else
    _log_warn_light "Skip WASM build step for ${DS}"
  fi

  _post_build "${DILLA_DIST_WASM_EXTISM_TPL_FOLDER}/" "${_DILLA_DS_TARGET}/" 'extism' 'gen_payload'
  _post_build "${DILLA_DIST_WASM_EXTISM_TPL_FOLDER}/browser" "${_DILLA_DS_TARGET}/browser" 'extism'
  _post_build "${DILLA_DIST_WASM_EXTISM_TPL_FOLDER}/node" "${_DILLA_DS_TARGET}/node" 'extism'

  __size=$(_size "${_DILLA_DS_TARGET}/${DS}.wasm")
  __size_dev=$(_size "${_DILLA_DS_TARGET}/${DS}_dev.wasm")
  _log_success "Extism created: ${__size}, DEV: ${__size_dev}"
}

__build_wasm_extism() {
  _log_notice "Build WASM Extism (wasm32-wasi)..."

  _cp_ds

  _log_notice "Cargo build..."
  _log_debug "DS=${DS} cargo build --target wasm32-wasi --release --no-default-features $_QUIET"

  cd "${DILLA_WASM_EXTISM_LIB}" && DS=${DS} cargo build --target wasm32-wasi --release --no-default-features $_QUIET
  cp -f "${__generated_wasm}" "${_DILLA_DS_TARGET}/${DS}.wasm"

  _log_notice "Cargo build DEV..."
  _log_debug "DS=${DS} cargo build --target wasm32-wasi --release $_QUIET"

  cd "${DILLA_WASM_EXTISM_LIB}" && DS=${DS} cargo build --target wasm32-wasi --release $_QUIET
  cp -f "${__generated_wasm}" "${_DILLA_DS_TARGET}/${DS}_dev.wasm"

  if _blank "${_NO_OPTIMIZATION}"; then
    _wasm_opt "${_DILLA_DS_TARGET}/${DS}.wasm"
    _wasm_opt "${_DILLA_DS_TARGET}/${DS}_dev.wasm"
  else
    _log_debug "Not optimization with wasm-opt"
  fi
}

###############################################################################
# Programs functions used by _base.sh
###############################################################################

_local_init() {
  # Target for DS build files.
  _DILLA_DS_TARGET="${DILLA_DIST_FOLDER}/${DS}/extism"
}

_local_info() {
  _log_info "${_ME} specific variables:"
  echo -e " DILLA_WASM_EXTISM_LIB:\t\t\t\t${DILLA_WASM_EXTISM_LIB:-"-"}"
  echo -e " --"
  echo -e " DILLA_WASM_EXTISM_URL:\t\t\t\t${DILLA_WASM_EXTISM_URL:-"-"}"
  echo -e " --"
  echo -e " _DILLA_DS_TARGET:\t\t\t\t${_DILLA_DS_TARGET:-"-"}"
  echo -e " _DILLA_DS_ROOT:\t\t\t\t${_DILLA_DS_ROOT:-"-"}"
}

_local_check() {
  if ! command -v cargo &>/dev/null; then
    _log_error "'cargo' could not be found, please install"
    _exit_1
  fi
  _ds_not_test
}

# shellcheck source=/dev/null
if [ -f "${_DIR}/_base.sh" ]; then source "${_DIR}/_base.sh"; else
  echo -e "\e[31m[Error] Missing _base.sh file!\e[0m"
  exit 1
fi
