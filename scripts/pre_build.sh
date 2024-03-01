#!/bin/bash
#
# Execute Dilla prebuilder image from DS sources to generate the DILLA_RUN_DS_FOLDER needed by Dilla.

_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# shellcheck source=/dev/null
if [ -f "${_DIR}/_init.sh" ]; then source "${_DIR}/_init.sh"; else
  echo -e "\e[31m[Error] Missing _init.sh file!\e[0m"
  exit 1
fi

_print_help() {
  cat <<HEREDOC
Execute Dilla prebuilder image from DS sources to generate the DILLA_RUN_DS_FOLDER needed by Dilla.

Commands:
  run              Prebuild DS for all under DILLA_INPUT_DIR
  all              Prebuild DS for all under DILLA_INPUT_DIR
  info             Get variables info used by this script

Options:
  --skip-check     Do not check schemas
  --skip-pull      Do not pull docker images
  -h --help        Display this help information

Usage:
  ${_ME} run [_DS_NAME_] --skip-check
HEREDOC
}

###############################################################################
# Specific script variables
###############################################################################

# Rust Docker image used for Docker run are set in env or default in _init.sh.

###############################################################################
# Script functions
###############################################################################

_all() {
  _init
  __pull_docker_images

  IFS=, read -ra values <<< "$DILLA_DS_NAMES"
  for _ds in "${values[@]}"; do DS="${_ds}" _run; done
}

_run() {
  _init
  __pull_docker_images
  __run
}

###############################################################################
# Programs internal functions
###############################################################################

__run() {
  local _input_dir=${DILLA_INPUT_DIR}/${DS}
  local _output_dir=${DILLA_OUTPUT_DIR}

  if _blank "${_input_dir}"; then
    _log_error "[${DS}] Missing input dir, set a path relative to ${DILLA_INPUT_DIR}/ as argument of this script"
    _exit_1
  fi

  _log_info "[${DS}] Pre-build Dilla DS from ${_input_dir} to ${_output_dir}/${DS}"

  if [ -d "${_output_dir}/${DS}" ]; then
    rm -rf "${_output_dir:?}/${DS}"
  fi

  if _blank "${_SKIP_CHECK_SCHEMAS-}"; then
    _log_notice "[${DS}] Check definitions and templates..."
    docker run --rm -t -v "${_input_dir}:/data/" "${DILLA_DOCKER_SCHEMAS}" run
  else
    _log_notice "[${DS}] Skip definitions and templates check!"
  fi

  _log_notice "[${DS}] Generate folders for '${DILLA_PREBUILD_CMD}'..."

  mkdir -p "${_output_dir}"
  chmod 777 "${_output_dir}"

  # shellcheck disable=SC2086
  docker run $_QUIET --rm -t -u "$(id -u):$(id -g)" -v "${_input_dir}:/data/input" -v "${_output_dir}:/data/output:rw" "${DILLA_DOCKER_PREBUILDER}" "${DILLA_PREBUILD_CMD}"

  if [[ -d "${_output_dir}/build" ]]; then
    mv "${_output_dir}/build" "${_output_dir}/${DS}"
  else
    _log_warn "Missing build folder!"
    # _exit_1
  fi

  if [[ -d "${_output_dir}/data" ]]; then
    if [[ -d "${_output_dir}/${DS}_data" ]]; then
      rm -rf "${_output_dir}/${DS}_data"
    fi
    mv "${_output_dir}/data" "${_output_dir}/${DS}_data"
  fi

  # Replace ds.rs if exist.
  if [[ -f "${_input_dir}/ds.rs" ]]; then
    _log_warn "[${DS}] Override ds.rs with existing file in ds src."
    cp "${_input_dir}/ds.rs" "${_output_dir}/${DS}/ds.rs"
  fi

  # Add examples if they are already in native json
  if [[ -d "${_input_dir}/examples" ]]; then
    _log_notice "[${DS}] Check native examples..."

    mkdir -p "${_output_dir}/${DS}/examples"

    local _has_json_file=false

    for _file in "${_input_dir}"/examples/*.json; do
      if [[ -f ${_file} ]]; then
        _has_json_file=true
        local filename="${_file##*/}"
        cp -f "${_file}" "${_output_dir}/${DS}/examples/${filename}"
      fi
    done

    if [[ ${_has_json_file} == "true" ]]; then
      _log_notice "[${DS}] Found native files, copied!"
    else
      _log_notice "[${DS}] No native json examples to copy."
    fi
  fi

  _log_success "[${DS}] Pre-build done!"
}

__pull_docker_images() {
  if [[ ${_SKIP_DOCKER_PULL} == 0 ]]; then
    _log_notice "Get Dilla Docker images..."
    docker login "${DILLA_DOCKER_REGISTRY}"
    docker pull --quiet "${DILLA_DOCKER_SCHEMAS}"
    docker pull --quiet "${DILLA_DOCKER_PREBUILDER}"
  else
    _log_warn_light "Skip docker pull, remove '--skip-pull' to pull last prebuilder image!"
  fi
}

###############################################################################
# Programs functions used by _base.sh
###############################################################################

_local_info() {
  _log_info "${_ME} specific variables:"
  echo -e " DILLA_INPUT_DIR:\t\t${DILLA_INPUT_DIR:-"-"}"
  echo -e " DILLA_OUTPUT_DIR:\t\t${DILLA_OUTPUT_DIR:-"-"}"
  echo -e " DILLA_PREBUILD_CMD:\t\t${DILLA_PREBUILD_CMD:-"-"}"
  echo -e " --"
  echo -e " _SKIP_DOCKER_PULL:\t${_SKIP_DOCKER_PULL:-"0"}"
  echo -e " _SKIP_CHECK_SCHEMAS:\t${_SKIP_CHECK_SCHEMAS:-"0"}"
}

# _local_init() {
  # @todo test CI
  # CI_PROJECT_DIR="${CI_PROJECT_DIR:=.}"

  # if _blank "${DILLA_INPUT_DIR-}"; then
  #   DILLA_INPUT_DIR="${CI_PROJECT_DIR}/"
  # fi

  # if _blank "${DILLA_OUTPUT_DIR-}"; then
  #   mkdir -p "${CI_PROJECT_DIR}/output"
  #   DILLA_OUTPUT_DIR="${CI_PROJECT_DIR}/output"
  # fi
# }

_local_check() {
  if ! command -v docker &>/dev/null; then
    _log_error "docker could not be found and is required for this script!"
    _error=1
  fi

  _ds_not_test
}

# shellcheck source=/dev/null
if [ -f "${_DIR}/_base.sh" ]; then source "${_DIR}/_base.sh"; else
  echo -e "\e[31m[Error] Missing _base.sh file!\e[0m"
  exit 1
fi
