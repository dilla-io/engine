#!/bin/bash
#
# Dilla WASM builder from Design System.
# shellcheck disable=SC2086

_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# shellcheck source=/dev/null
if [ -f "${_DIR}/_init.sh" ]; then source "${_DIR}/_init.sh"; else
  echo -e "\e[31m[Error] Missing _init.sh file!\e[0m"
  exit 1
fi

_print_help() {
  cat <<HEREDOC
Dilla WASM CLI builder from Design System.

Argument:
  Dilla Design System name

Commands:
  run              Build a local DS Bin
  all              Build all Design System Bins
  info | i         Get variables info used by this script

Options:
  -v --verbose     Run script with more output
  -h --help        Display this help information

Usage:
  ${_ME} run [_DS_NAME_]
HEREDOC
}

###############################################################################
# Specific script variables
###############################################################################

###############################################################################
# Script functions
###############################################################################

_all() {
  IFS=, read -ra values <<< "$DILLA_DS_NAMES" 
  for _ds in "${values[@]}"; do DS="${_ds}" _run; done
}

_run() {
  _valid_ds
  _log_info "Generate BIN CLI for: ${DS} v${VERSION}..."

  _cp_ds

  _log_debug "DS=${DS} cargo build --target x86_64-unknown-linux-gnu --release --package dilla-* $_QUIET"

  DS=${DS} cargo build --target x86_64-unknown-linux-gnu --release --package dilla-* $_QUIET

  mkdir -p "${DILLA_DIST_FOLDER}"/bin
  _log_notice "Cargo build..."
  cp "${_DILLA_ROOT_DIR}/target/x86_64-unknown-linux-gnu/release/dilla-cli" "${DILLA_DIST_FOLDER}/bin/${DS}"

  __size=$(_size "${DILLA_DIST_FOLDER}/bin/${DS}")
  _log_success "BIN DEV created: ${__size}"
}

###############################################################################
# Programs functions used by _base.sh
###############################################################################

_local_info() {
  _log_info "${_ME} specific variables:"
  echo -e " DILLA_DIST_FOLDER:\t\t${DILLA_DIST_FOLDER:-"-"}"
  echo -e " DILLA_LIB_FOLDER:\t\t${DILLA_LIB_FOLDER:-"-"}"
  echo -e " DILLA_RUN_DS_FOLDER:\t\t${DILLA_RUN_DS_FOLDER:-"-"}"
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
