#!/bin/bash
#
# Dilla commands related to design systems.
# shellcheck disable=SC2115,SC2164

_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# shellcheck source=/dev/null
if [ -f "${_DIR}/_init.sh" ]; then source "${_DIR}/_init.sh"; else
  echo -e "\e[31m[Error] Missing _init.sh file!\e[0m"
  exit 1
fi

_print_help() {
  cat <<HEREDOC
Dilla commands related to design systems.

Argument:
  Dilla Design System name

Commands:
  clone            Clone repositories of Dilla DS
  clone_all        Clone ALL repositories of Dilla from DILLA_DS_NAMES
  update           Update ALL repositories of Dilla DS
  clone_url        Clone a given repository containing a Dilla Design System as root
  info | i         Get variables info used by this script

Options:
  -name            Required for clone_url name of Design System
  -repository      Required for clone_url https repository of Design System
  -h --help        Display this help information

Usage:
  ${_ME} clone
  ${_ME} clone_url -name [_DS_NAME_] -repository [_DS_HTTPS_URL_]
  ${_ME} update
HEREDOC
}

###############################################################################
# Script functions
###############################################################################

_clone() {
  _init
  __clone_ds
}

_clone_all() {
  _init
  IFS=, read -ra values <<< "$DILLA_DS_NAMES"
  for _ds in "${values[@]}"; do DS="${_ds}" __clone_ds; done
}

_clone_url() {
  _init

  _error=0
  if _blank "${DS-}"; then
    _log_error "Missing variable DS"
    _error=1
  fi
  if _blank "${_REPO-}"; then
    _log_error "Missing variable _REPO"
    _error=1
  fi

  if [[ ${_error} = 1 ]]; then
    _exit_1
  fi

  _log_info "Clone DS: ${DS} into ${DILLA_INPUT_DIR}/${DS}..."
  if [[ -d "${DILLA_INPUT_DIR}/${DS}" ]]; then
    rm -rf "${DILLA_INPUT_DIR}/${DS}"
  fi

  cd "${DILLA_INPUT_DIR}/"
  git clone --quiet "${_REPO}"

  _log_success "Repository ${_REPO} cloned successfully in ${DILLA_INPUT_DIR}/${DS}!"
}

_update() {
  _init
  IFS=, read -ra values <<< "$DILLA_DS_NAMES"
  for _ds in "${values[@]}"
  do
    _log_info "Update repository DS: ${_ds}..."
    if [ -d "${DILLA_INPUT_DIR}/${_ds}" ]; then
      git -C "${DILLA_INPUT_DIR}/${_ds}" pull
    else
      _log_warn "No repo for ${_ds}, did you run clone?"
    fi
  done
}

###############################################################################
# Programs internal functions
###############################################################################

__clone_ds() {
  _ds_not_test
  _token_req

  _log_info "Clone DS: ${DS} into ${DILLA_INPUT_DIR}/${DS}..."

  if [[ -d "${DILLA_INPUT_DIR}/${DS}" ]]; then
    rm -rf "${DILLA_INPUT_DIR}/${DS}"
  fi

  cd "${DILLA_INPUT_DIR}/"
  git clone --quiet "https://oauth2:${GITLAB_TOKEN}@gitlab.com/dilla-io/ds/${DS}.git"

  _log_success "Repository for ${DS} cloned successfully in ${DILLA_INPUT_DIR}/${DS}!"
}

###############################################################################
# Programs functions used by _base.sh
###############################################################################

_local_info() {
  _log_info "${_ME} specific variables:"
  echo -e " DILLA_INPUT_DIR:\t\t${DILLA_INPUT_DIR:-"-"}"
  # echo -e " DILLA_DS_VERSION:\t\t${DILLA_DS_VERSION:-"-"}"
  echo -e " DILLA_DS_PKG_URL:\t\t${DILLA_DS_PKG_URL:-"-"}"
  echo -e " --"
  echo -e " DILLA_PREBUILDER_PID:${DILLA_PREBUILDER_PID:-"-"}"
}

_local_init() {
  if _present "${DILLA_INPUT_DIR-}"; then
    if ! [[ -d "${DILLA_INPUT_DIR}/" ]]; then
      mkdir -p "${DILLA_INPUT_DIR}/"
    fi
  fi
}

_local_check() {
  if _blank "${DILLA_PREBUILDER_PID-}"; then
    _log_error "Missing variable DILLA_PREBUILDER_PID"
    _error=1
  fi

  if ! command -v git &>/dev/null; then
    _log_error "'git' could not be found, please install"
    _error=1
  fi

  if ! command -v curl &>/dev/null; then
    _log_error "'curl' could not be found, please install"
    _error=1
  fi

  _ds_not_test
}

# shellcheck source=/dev/null
if [ -f "${_DIR}/_base.sh" ]; then source "${_DIR}/_base.sh"; else
  echo -e "\e[31m[Error] Missing _base.sh file!\e[0m"
  exit 1
fi
