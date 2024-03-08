#!/bin/bash
#
# Dilla WASM Component builder from Design System.
# shellcheck disable=SC2086

_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# shellcheck source=/dev/null
if [ -f "${_DIR}/_init.sh" ]; then source "${_DIR}/_init.sh"; else
  echo -e "\e[31m[Error] Missing _init.sh file!\e[0m"
  exit 1
fi

_print_help() {
  cat <<HEREDOC
Dilla WASM Component builder from Design System.

Argument:
  Dilla Design System name

Commands:
  run              Build a single Design System WASM Component
  all              Build all Design System WASM Component
  docker_run       Run build a single DS WASM Component with a Docker image
  docker_all       Run build WASM Component with a Docker image
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
HEREDOC
}

###############################################################################
# Specific script variables
###############################################################################

_IS_DOCKER=0
__generated_wasm="${_DILLA_ROOT_DIR}/target/wasm32-wasi/release/wasm-component.wasm"

###############################################################################
# Script functions
###############################################################################

_run() {
  _init
  _valid_ds
  __build_wasm
}

_all() {
  IFS=, read -ra values <<<"$DILLA_DS_NAMES"
  for _ds in "${values[@]}"; do DS="${_ds}" _run; done
}

_docker_run() {
  _docker_run_build "co"
}

_docker_all() {
  _docker_build_all "co"
}

###############################################################################
# Programs internal functions
###############################################################################

__build_wasm() {
  _log_info "Generate WASM Component for: ${DS} v${VERSION}..."
  if ! ((_NO_BUILD)); then
    __prepare_dir "${_DILLA_DS_TARGET}"
    __build_wasm_component
  else
    _log_warn_light "Skip WASM build step for ${DS}"
  fi

  __build_wasm_jco

  _post_build "${DILLA_DIST_WASM_COMPONENT_TPL_FOLDER}/" "${_DILLA_DS_TARGET}/" 'component' 'gen_payload'

  __prepare_dir "${_DILLA_DS_TARGET}/browser"
  _post_build "${DILLA_DIST_WASM_COMPONENT_TPL_FOLDER}/browser" "${_DILLA_DS_TARGET}/browser" 'component'

  __prepare_dir "${_DILLA_DS_TARGET}/node"
  _post_build "${DILLA_DIST_WASM_COMPONENT_TPL_FOLDER}/node" "${_DILLA_DS_TARGET}/node" 'component'

  __size=$(_size "${_DILLA_DS_TARGET}/${DS}.core.wasm")
  __size_dev=$(_size "${_DILLA_DS_TARGET}/${DS}_dev.core.wasm")
  _log_success "Component created: ${__size}, DEV: ${__size_dev}"
}

__build_wasm_component() {
  _log_notice "Build WASM Component (cargo component [wasm32-wasi])"

  __prepare_dir "${_DILLA_DS_TARGET}/wasm"
  _cp_ds

  _log_debug "$(cargo component --version)"
  _log_debug "DS=${DS} cargo component build -p wasm-component -r --no-default-features $_FEATURES $_QUIET"

  _log_notice "Cargo build..."

  DS=${DS} cargo component build -p wasm-component -r --no-default-features $_FEATURES $_QUIET

  cp "${__generated_wasm}" "${_DILLA_DS_TARGET}/wasm/${DS}.wasm"

  _log_notice "Cargo build DEV..."

  _log_debug "DS=${DS} cargo component build -p wasm-component -r -F \"describer\" $_FEATURES $_QUIET"
  _log_debug "cargo component build -p wasm-component -r -F \"describer\" $_QUIET"

  DS=${DS} cargo component build -p wasm-component -r -F "describer" $_FEATURES $_QUIET

  cp "${__generated_wasm}" "${_DILLA_DS_TARGET}/wasm/${DS}_dev.wasm"
}

__build_wasm_jco() {
  _log_notice "Transpile WASM with JCO and Binaryen..."

  if [ ! -d "${DILLA_DIST_FOLDER}/node_modules" ]; then
    _log_notice "Instal JCO and Binaryen..."
    if [ ! -f "${DILLA_DIST_FOLDER}/package.json" ]; then
      cd "${DILLA_DIST_FOLDER}" && npm init -y -f -s
    fi
    _log_debug "npm install @bytecodealliance/jco binaryen -s"
    cd "${DILLA_DIST_FOLDER}" && npm install @bytecodealliance/jco binaryen -s
  fi

  # local _opt='--no-typescript'
  local _opt=''

  # Default optimization with jco:
  # '-O1 --low-memory-unused --enable-bulk-memory'
  if _blank "${_NO_OPTIMIZATION}"; then
    _opt+=' --valid-lifting-optimization --minify --optimize -- -Os --low-memory-unused --enable-bulk-memory'
  else
    _log_debug "Not optimization, tracing enabled"
    _opt+=" --tracing" # Dev tracing errors
  fi

  _log_notice "jco transpile: ${DS}, opt:${_opt} ..."
  _log_debug "npx jco transpile ${_QUIET} --name ${DS} ${_DILLA_DS_TARGET}/wasm/${DS}.wasm --out-dir ${_DILLA_DS_TARGET} $_opt"

  cd "${DILLA_DIST_FOLDER}" &&
    eval npx jco transpile ${_QUIET} --name "${DS}" "${_DILLA_DS_TARGET}/wasm/${DS}.wasm" --out-dir "${_DILLA_DS_TARGET}" $_opt &&
    mv "${_DILLA_DS_TARGET}/${DS}.js" "${_DILLA_DS_TARGET}/${DS}.mjs" &&
    eval npx jco transpile ${_QUIET} --name "${DS}_dev" "${_DILLA_DS_TARGET}/wasm/${DS}_dev.wasm" --out-dir "${_DILLA_DS_TARGET}/tmp" $_opt &&
    mv "${_DILLA_DS_TARGET}/tmp/${DS}_dev.core.wasm" "${_DILLA_DS_TARGET}/" &&
    mv "${_DILLA_DS_TARGET}/tmp/${DS}_dev.core2.wasm" "${_DILLA_DS_TARGET}/" &&
    mv "${_DILLA_DS_TARGET}/tmp/${DS}_dev.js" "${_DILLA_DS_TARGET}/${DS}_dev.mjs" &&
    rm -rf "${_DILLA_DS_TARGET}/tmp"

  _log_success_light "WASM Component created in ${_DILLA_DS_TARGET}!"
}

###############################################################################
# Programs functions used by _base.sh
###############################################################################

_local_init() {
  # Target for DS build files.
  _DILLA_DS_TARGET="${DILLA_DIST_FOLDER}/${DS}/component"
}

_local_info() {
  _log_info "${_ME} specific variables:"
  echo -e " _DO_PATCH:\t\t\t\t\t${_DO_PATCH:-"-"}"
  echo -e " _PATCH_SRC:\t\t\t\t\t${_PATCH_SRC:-"-"}"
  echo -e " DILLA_WASM_COMPONENT_LIB:\t\t\t${DILLA_WASM_COMPONENT_LIB:-"-"}"
  echo -e " --"
  echo -e " DILLA_DIST_WASM_COMPONENT_TPL_FOLDER:\t\t${DILLA_DIST_WASM_COMPONENT_TPL_FOLDER:-"-"}"
  echo -e " --"
  echo -e " DILLA_DATA_FILE_SRC_TESTS:\t\t\t\t${DILLA_DATA_FILE_SRC_TESTS:-"-"}"
  echo -e " _DATA_FILE_SRC_EXAMPLES:\t\t\t${_DATA_FILE_SRC_EXAMPLES:-"-"}"
  echo -e " _DILLA_DS_TARGET:\t\t\t\t${_DILLA_DS_TARGET:-"-"}"
}

_local_check() {
  if ! command -v cargo component &>/dev/null; then
    _log_error "'cargo component' could not be found, please install"
    _exit_1
  fi
  _ds_not_test
}

# shellcheck source=/dev/null
if [ -f "${_DIR}/_base.sh" ]; then source "${_DIR}/_base.sh"; else
  echo -e "\e[31m[Error] Missing _base.sh file!\e[0m"
  exit 1
fi
