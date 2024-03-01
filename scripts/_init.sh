#!/usr/bin/env bash
#
# Init Dilla scripts env, variables and parsing.
#
# Bash Boilerplate: https://github.com/alphabetum/bash-boilerplate
#
# Copyright (c) 2015 William Melody • hi@williammelody.com

# Short form: set -u
set -o nounset

# Short form: set -e
set -o errexit

# Print a helpful message if a pipeline with non-zero exit code causes the
# script to exit as described above.
trap 'echo "Aborting due to errexit on line $LINENO. Exit code: $?" >&2' ERR

# Allow the above trap be inherited by all functions in the script.
#
# Short form: set -E
set -o errtrace

set -o pipefail

# Set $IFS to only newline and tab.
#
# http://www.dwheeler.com/essays/filenames-in-shell.html
IFS=$'\n\t'

###############################################################################
# Environment
###############################################################################

# $_ME
#
# Set to the program's basename.
_ME=$(basename "${0}")

# $_DIR
#
# Set to the program's base dir.
_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# $_DILLA_ROOT_DIR
#
# Set to the program's root dir, assume scripts are in scripts/ folder.
_DILLA_ROOT_DIR="${_DIR/\/scripts/}"

###############################################################################
# Dilla globals
###############################################################################

_IS_CI=0
_ENV_DEFAULT=".env.default"
_LOCAL_ENV_LOADED=

if [ -n "${CI-}" ]; then
  echo -e "\e[33m[Notice] Detected CI env\e[0m"
  _IS_CI=1
  if [ -n "${CI_PROJECT_NAME:-}" ]; then
    # shellcheck disable=SC2034
    DS="${CI_PROJECT_NAME}"
  fi

  if [ -n "${CI_PROJECT_ID:-}" ]; then
    # shellcheck disable=SC2034
    DILLA_PROJECT_ID="${CI_PROJECT_ID}"
  fi

  if [ -n "${CI_COMMIT_TAG:-}" ]; then
    VERSION="${CI_COMMIT_TAG}"
  else
    VERSION="1.0.0"
  fi

  _TESTS_FOLDER="crates/dilla-renderer/tests"

  echo -e "\tDS: ${DS}"
  echo -e "\tDILLA_PROJECT_ID: ${DILLA_PROJECT_ID}"
  echo -e "\tVERSION: ${VERSION}"
  echo -e "\t_DILLA_ROOT_DIR: ${_DILLA_ROOT_DIR}"
  # _DILLA_ROOT_DIR="/app"
  # echo -e "\t_DILLA_ROOT_DIR (forced): ${_DILLA_ROOT_DIR}"
else
  if [ -f "${_DILLA_ROOT_DIR}/.env" ]; then
    # First load default values.
    if [ -f "${_DILLA_ROOT_DIR}/${_ENV_DEFAULT}" ]; then
      # shellcheck source=/dev/null
      source "${_DILLA_ROOT_DIR}/${_ENV_DEFAULT}"
    fi
    # shellcheck source=/dev/null
    source "${_DILLA_ROOT_DIR}/.env"
    _LOCAL_ENV_LOADED=1
  elif [ -f "${_DILLA_ROOT_DIR}/${_ENV_DEFAULT}" ]; then
    # shellcheck source=/dev/null
    source "${_DILLA_ROOT_DIR}/${_ENV_DEFAULT}"
  fi
fi

if [ -z "${DS-}" ]; then
  DS="__NAME_TO_SET__"
fi

if [ -z "${VERSION-}" ]; then
  # Default is to use current tag.
  # @todo check if needed as we use --version for publish.sh up
  if [[ -d "${_DILLA_ROOT_DIR}/.git" ]]; then
    __tag=$(git describe --abbrev=0 --tags)
    VERSION="${__tag/v/}"
  else
    VERSION="1.0.0"
  fi
fi

if [ -z "${CI-}" ]; then
  # Dilla variables relative to ROOT.
  DILLA_INPUT_DIR="${_DILLA_ROOT_DIR}/${DILLA_INPUT_DIR:-}"
  DILLA_OUTPUT_DIR="${_DILLA_ROOT_DIR}/${DILLA_OUTPUT_DIR}"
  DILLA_DIST_FOLDER="${_DILLA_ROOT_DIR}/${DILLA_DIST_FOLDER}"

  DILLA_DIST_WASM_BINDGEN_TPL_FOLDER="${_DILLA_ROOT_DIR}/${DILLA_DIST_WASM_BINDGEN_TPL_FOLDER}"
  DILLA_DIST_WASM_COMPONENT_TPL_FOLDER="${_DILLA_ROOT_DIR}/${DILLA_DIST_WASM_COMPONENT_TPL_FOLDER}"
  DILLA_DIST_WASM_EXTISM_TPL_FOLDER="${_DILLA_ROOT_DIR}/${DILLA_DIST_WASM_EXTISM_TPL_FOLDER}"

  DILLA_RUN_DS_FOLDER="${_DILLA_ROOT_DIR}/${DILLA_RUN_DS_FOLDER}"

  DILLA_LIB_FOLDER="${_DILLA_ROOT_DIR}/${DILLA_LIB_FOLDER}"
  DILLA_WASM_BINDGEN_LIB="${_DILLA_ROOT_DIR}/${DILLA_WASM_BINDGEN_LIB}"
  DILLA_WASM_COMPONENT_LIB="${_DILLA_ROOT_DIR}/${DILLA_WASM_COMPONENT_LIB}"
  DILLA_WASM_EXTISM_LIB="${_DILLA_ROOT_DIR}/${DILLA_WASM_EXTISM_LIB}"
fi

_TESTS_FOLDER="${_DILLA_ROOT_DIR}/${_TESTS_FOLDER}"

# Flag to set this script loaded.
_INIT_DONE=1
