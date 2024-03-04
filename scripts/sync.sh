#!/bin/bash
#
# Dilla commands related to remote sync.

_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# shellcheck source=/dev/null
if [[ -f "${_DIR}/_init.sh" ]]; then source "${_DIR}/_init.sh"; else
  echo -e "\e[31m[Error] Missing _init.sh file!\e[0m"
  exit 1
fi

_print_help() {
  cat <<HEREDOC
Dilla commands related to remote sync based on .env variables.

Argument:
  Dilla Design System name

Commands:
  data             Send Data on data.dilla.io from local DILLA_RUN_DS_FOLDER
  doc              Send Dilla doc on doc.dilla.io from local generated doc (just doc)
  bg               [bindgen] Current DS Bindgen WASM on data.dilla.io
  bg_all           [bindgen] Send ALL DS Bindgen
  co               [component] Current DS Component WASM on data.dilla.io
  co_all           [component] Send ALL DS Component
  ex               [extism] Current DS Extism WASM on data.dilla.io
  ex_all           [extism] Send ALL DS Extism
  all_ds           Send all ds generated!
  all              Send all!
  up_exp           [TMP] Run remote script /home/debian/web/update_explorer.sh
  info | i         Get variables info used by this script

Options:
  -h --help        Display this help information

Usage:
  ${_ME} data
  ${_ME} doc
HEREDOC
}

###############################################################################
# Script functions
###############################################################################

_all_ds() {
  _bg_all
  _ex_all
  _co_all
}

_all() {
  _all_ds
  _data
  _doc
}

# Extism
_ex() {
  _init
  local _src_dir="${DILLA_DIST_FOLDER}/${DS}/extism"
  local _remote_dir="${DILLA_SYNC_REMOTE_WASM_EXTISM_PATH}/${DS}"
  _send_remote "${_src_dir}" "${DILLA_SYNC_REMOTE}" "${_remote_dir}" "${DS}"
}

_ex_all() {
  _init
  IFS=, read -ra values <<< "$DILLA_DS_NAMES"
  for _ds in "${values[@]}"; do DS="${_ds}" _ex; done
}

# Bindgen
_co() {
  _init
  local _src_dir="${DILLA_DIST_FOLDER}/${DS}/component"
  local _remote_dir="${DILLA_SYNC_REMOTE_WASM_COMPONENT_PATH}/${DS}"
  _send_remote "${_src_dir}" "${DILLA_SYNC_REMOTE}" "${_remote_dir}" "${DS}"
}

_co_all() {
  _init
  IFS=, read -ra values <<< "$DILLA_DS_NAMES"
  for _ds in "${values[@]}"; do DS="${_ds}" _co; done
}

# Bindgen
_bg() {
  _init
  local _src_dir="${DILLA_DIST_FOLDER}/${DS}/bindgen"
  local _remote_dir="${DILLA_SYNC_REMOTE_WASM_BINDGEN_PATH}/${DS}"
  _send_remote "${_src_dir}" "${DILLA_SYNC_REMOTE}" "${_remote_dir}" "${DS}"
}

_bg_all() {
  _init
  IFS=, read -ra values <<< "$DILLA_DS_NAMES"
  for _ds in "${values[@]}"; do DS="${_ds}" _bg; done
}

_data() {
  _init

  if _blank "${DILLA_SYNC_REMOTE_DATA-}"; then
    _log_error "Missing variable DILLA_SYNC_REMOTE_DATA"
    _error=1
  fi

  if ! _blank "${_ARGS-}"; then
    local _src_dir="${DILLA_RUN_DS_FOLDER}/${DS}_data"
    local _remote_dir="${DILLA_SYNC_REMOTE_DATA}/${DS}"
    _send_remote "${_src_dir}" "${DILLA_SYNC_REMOTE}" "${_remote_dir}" "${DS}_data"
  else
    IFS=, read -ra values <<< "$DILLA_DS_NAMES"
    for _ds in "${values[@]}"; do
      if [[ -d "${DILLA_RUN_DS_FOLDER}/${_ds}_data" ]]; then
        local _src_dir="${DILLA_RUN_DS_FOLDER}/${_ds}_data"
        local _remote_dir="${DILLA_SYNC_REMOTE_DATA}/${_ds}"
        _send_remote "${_src_dir}" "${DILLA_SYNC_REMOTE}" "${_remote_dir}" "${_ds}_data"
      fi
    done
  fi
}

_doc() {
  _init
  local _src_dir="${_DILLA_ROOT_DIR}/target/doc"
  if ! [ -d "${_src_dir}" ]; then
    _log_error "Missing doc, run 'just doc' to generate"
    return
  fi
  local _remote_dir="${DILLA_SYNC_REMOTE_DOC_PATH}/"
  _send_remote "${_src_dir}" "${DILLA_SYNC_REMOTE}" "${_remote_dir}" "doc"
}

_up_exp() {
  _init
  ssh -4 "${DILLA_SYNC_REMOTE}" bash -c "${DILLA_SYNC_REMOTE_ROOT}/web/update_explorer.sh"
}

###############################################################################
# Programs internal functions
###############################################################################

###############################################################################
# Programs functions used by _base.sh
###############################################################################

_local_info() {
  _log_info "${_ME} specific variables:"
  echo -e " DILLA_SYNC_REMOTE:\t\t${DILLA_SYNC_REMOTE:-"-"}"
  echo -e " DILLA_SYNC_REMOTE_ROOT:\t${DILLA_SYNC_REMOTE_ROOT:-"-"}"
  echo -e " DILLA_SYNC_REMOTE_DATA:\t${DILLA_SYNC_REMOTE_DATA:-"-"}"
  echo -e " DILLA_SYNC_REMOTE_DOC_PATH:\t${DILLA_SYNC_REMOTE_DOC_PATH:-"-"}"
  echo -e "----"
  echo -e " DILLA_SYNC_REMOTE_WASM_BINDGEN_PATH:\t${DILLA_SYNC_REMOTE_WASM_BINDGEN_PATH:-"-"}"
  echo -e " DILLA_DIST_FOLDER:\t${DILLA_DIST_FOLDER:-"-"}"
  echo -e " DILLA_DIST_WASM_BG_PLG_FOLDER:\t${DILLA_DIST_WASM_BG_PLG_FOLDER:-"-"}"
  echo -e " DILLA_SYNC_REMOTE_WASM_BINDGEN_PLAYGROUND_PATH:\t${DILLA_SYNC_REMOTE_WASM_BINDGEN_PLAYGROUND_PATH:-"-"}"
  echo -e "----"
  echo -e " DILLA_SYNC_REMOTE_WASM_COMPONENT_PATH:\t${DILLA_SYNC_REMOTE_WASM_COMPONENT_PATH:-"-"}"
  echo -e "----"
  echo -e " DILLA_SYNC_REMOTE_WASM_EXTISM_PATH:\t${DILLA_SYNC_REMOTE_WASM_EXTISM_PATH:-"-"}"
}

_local_check() {
  _ds_not_test
}

# shellcheck source=/dev/null
if [[ -f "${_DIR}/_base.sh" ]]; then source "${_DIR}/_base.sh"; else
  echo -e "\e[31m[Error] Missing _base.sh file!\e[0m"
  exit 1
fi
