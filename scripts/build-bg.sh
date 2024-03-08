#!/bin/bash
#
# Dilla WASM Bindgen builder from Design System.
# shellcheck disable=SC2086

_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# shellcheck source=/dev/null
if [ -f "${_DIR}/_init.sh" ]; then source "${_DIR}/_init.sh"; else
  echo -e "\e[31m[Error] Missing _init.sh file!\e[0m"
  exit 1
fi

_print_help() {
  cat <<HEREDOC
Dilla WASM Bindgen builder from Design System.

Argument:
  Dilla Design System name

Commands:
  run              Build a single Design System WASM Bindgen
  all              Build all Design System  WASM Bindgen
  docker_run       Run build a single DS WASM Bindgen with a Docker image
  docker_all       Run build WASM Bindgen with a Docker image
  info | i         Get variables info used by this script

Options:
  -f --features    Custom Cargo build features flag
  --skip-pull      Skip Docker pull for run_docker* commands
  --no-optim       Skip build optimization
  -v --verbose     Run script with more output
  -h --help        Display this help information

Usage:
  ${_ME} run [_DS_NAME_]
  ${_ME} all
  ${_ME} docker_run [_DS_NAME_]
  ${_ME} docker_run [_DS_NAME_] --skip-pull
  ${_ME} docker_all
HEREDOC
}

###############################################################################
# Specific script variables
###############################################################################

__generated_wasm="${_DILLA_ROOT_DIR}/target/wasm32-unknown-unknown/release/wasm_bindgen_dilla.wasm"

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
  _docker_run_build "bg"
}

_docker_all() {
  _docker_build_all "bg"
}

###############################################################################
# Programs internal functions
###############################################################################

__build_wasm() {
  _log_info "Generate WASM Bindgen for: ${DS} v${VERSION}..."
  if ! ((_NO_BUILD)); then
    __prepare_dir "${_DILLA_DS_TARGET}"
    __build_wasm_wasm32
  else
    _log_warn_light "Skip WASM build step for ${DS}"
  fi

  __build_wasm_bindgen

  _post_build "${DILLA_DIST_WASM_BINDGEN_TPL_FOLDER}/" "${_DILLA_DS_TARGET}/" 'bindgen' 'gen_payload'
  _post_build "${DILLA_DIST_WASM_BINDGEN_TPL_FOLDER}/browser" "${_DILLA_DS_TARGET}/browser" 'bindgen'
  _post_build "${DILLA_DIST_WASM_BINDGEN_TPL_FOLDER}/node" "${_DILLA_DS_TARGET}/node" 'bindgen'

  __size=$(_size "${_DILLA_DS_TARGET}/browser/${DS}_bg.wasm")
  __size_dev=$(_size "${_DILLA_DS_TARGET}/browser/${DS}_dev_bg.wasm")
  _log_success "Bindgen browser created: ${__size}, DEV: ${__size_dev}"

  __size=$(_size "${_DILLA_DS_TARGET}/node/${DS}_bg.wasm")
  __size_dev=$(_size "${_DILLA_DS_TARGET}/node/${DS}_dev_bg.wasm")
  _log_success "Bindgen node created: ${__size}, DEV: ${__size_dev}"
}

__build_wasm_wasm32() {
  _log_notice "Build WASM Bindgen (wasm32-unknown-unknown)..."

  __prepare_dir "${_DILLA_DS_TARGET}/wasm"
  _cp_ds

  _log_debug "DS=${DS} cargo build -p wasm-bindgen-dilla --target wasm32-unknown-unknown -r --no-default-features $_FEATURES $_QUIET"

  _log_notice "Cargo build..."
  DS=${DS} cargo build -p wasm-bindgen-dilla --target wasm32-unknown-unknown -r --no-default-features $_FEATURES $_QUIET
  cp "${__generated_wasm}" "${_DILLA_DS_TARGET}/wasm/${DS}.wasm"

  _log_notice "Cargo build DEV..."
  DS=${DS} cargo build -p wasm-bindgen-dilla --target wasm32-unknown-unknown -r -F "describer" $_FEATURES $_QUIET
  cp "${__generated_wasm}" "${_DILLA_DS_TARGET}/wasm/${DS}_dev.wasm"
}

__build_wasm_bindgen() {
  local _opt=''
  if ! _blank "${_NO_OPTIMIZATION}"; then
    _log_debug "Not optimization, debug mode"
    _opt="--debug" # Dev errors and DWARF
  fi

  _log_notice "wasm-bindgen: ${DS} in ${_DILLA_DS_TARGET}..."

  _log_debug "$(wasm-bindgen --version)"
  _log_debug "wasm-bindgen ${_DILLA_DS_TARGET}/wasm/${DS}.wasm --out-dir ${_DILLA_DS_TARGET}/browser --out-name ${DS} --target web $_opt"

  __prepare_dir "${_DILLA_DS_TARGET}/browser"
  wasm-bindgen "${_DILLA_DS_TARGET}/wasm/${DS}.wasm" --out-dir "${_DILLA_DS_TARGET}/browser" --out-name "${DS}" --target web $_opt
  wasm-bindgen "${_DILLA_DS_TARGET}/wasm/${DS}_dev.wasm" --out-dir "${_DILLA_DS_TARGET}/browser" --out-name "${DS}_dev" --target web $_opt

  __prepare_dir "${_DILLA_DS_TARGET}/node"
  wasm-bindgen "${_DILLA_DS_TARGET}/wasm/${DS}.wasm" --out-dir "${_DILLA_DS_TARGET}/node" --out-name "${DS}" --target nodejs $_opt
  wasm-bindgen "${_DILLA_DS_TARGET}/wasm/${DS}_dev.wasm" --out-dir "${_DILLA_DS_TARGET}/node" --out-name "${DS}_dev" --target nodejs $_opt

  if _blank "${_NO_OPTIMIZATION}"; then
    local _opt='-Os --low-memory-unused --enable-bulk-memory'
    _wasm_opt "${_DILLA_DS_TARGET}/browser/${DS}_bg.wasm" $_opt
    _wasm_opt "${_DILLA_DS_TARGET}/browser/${DS}_dev_bg.wasm" $_opt
    _wasm_opt "${_DILLA_DS_TARGET}/node/${DS}_bg.wasm" $_opt
    _wasm_opt "${_DILLA_DS_TARGET}/node/${DS}_dev_bg.wasm" $_opt
  else
    _log_debug "Not optimization"
  fi
}

###############################################################################
# Programs functions used by _base.sh
###############################################################################

_local_check() {
  if ! command -v cargo &>/dev/null; then
    _log_error "'cargo' could not be found, please install"
    _exit_1
  fi

  if ! command -v wasm-bindgen &>/dev/null; then
    _log_error "'wasm-bindgen' could not be found, please install"
    _exit_1
  fi

  if _blank "${_NO_OPTIMIZATION}"; then
    if ! command -v wasm-opt &>/dev/null; then
      _log_warn "'wasm-opt' could not be found, skip size optimization, please install to fix this"
    fi
  fi
  _ds_not_test
}

_local_init() {
  # Target for DS build files.
  _DILLA_DS_TARGET="${DILLA_DIST_FOLDER}/${DS}/bindgen"
}

_local_info() {
  _log_info "${_ME} specific variables:"
  echo -e " DILLA_WASM_BINDGEN_LIB:\t\t\t\t${DILLA_WASM_BINDGEN_LIB:-"-"}"
  echo -e " DILLA_JS_HELPER_URL:\t\t\t\t${DILLA_JS_HELPER_URL:-"-"}"
  echo -e " --"
  echo -e " DILLA_DIST_WASM_BINDGEN_TPL_FOLDER:\t\t${DILLA_DIST_WASM_BINDGEN_TPL_FOLDER:-"-"}"
  echo -e " DILLA_DIST_WASM_BG_PLG_FOLDER:\t${DILLA_DIST_WASM_BG_PLG_FOLDER:-"-"}"
  echo -e " --"
  echo -e " DILLA_DATA_FILE_SRC_TESTS:\t\t\t\t${DILLA_DATA_FILE_SRC_TESTS:-"-"}"
  echo -e " _DATA_FILE_SRC_EXAMPLES:\t\t\t${_DATA_FILE_SRC_EXAMPLES:-"-"}"
  echo -e " _DILLA_DS_TARGET:\t\t\t\t${_DILLA_DS_TARGET:-"-"}"
  echo -e " _DILLA_DS_ROOT:\t\t\t\t${_DILLA_DS_ROOT:-"-"}"
  echo -e " --"
  if ((_SKIP_DOCKER_PULL)); then
    echo -e " _SKIP_DOCKER_PULL:\t\t\t\tTrue"
  else
    echo -e " _SKIP_DOCKER_PULL:\t\t\t\tFalse"
  fi
  echo -e " --"
  echo -e " DILLA_DOCKER_RUST:\t\t\t\t${DILLA_DOCKER_RUST:-"-"}"
}

# shellcheck source=/dev/null
if [ -f "${_DIR}/_base.sh" ]; then source "${_DIR}/_base.sh"; else
  echo -e "\e[31m[Error] Missing _base.sh file!\e[0m"
  exit 1
fi
