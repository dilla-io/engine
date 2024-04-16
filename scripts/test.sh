#!/bin/bash
#
# Dilla WASM builder from Design System with WASM-bindgen.
# shellcheck disable=SC2086,SC2209,SC2129

_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# shellcheck source=/dev/null
if [ -f "${_DIR}/_init.sh" ]; then source "${_DIR}/_init.sh"; else
  echo -e "\e[31m[Error] Missing _init.sh file!\e[0m"
  exit 1
fi

# run_docker       Run tests for Design System as argument within a Docker image
# run_docker_all   Run all tests for Dilla Design Systems defined as DILLA_DS_NAMES within a Docker
# gen_all          Generate HTML result files for all tests under run_ds_src

_print_help() {
  cat <<HEREDOC
Dilla helper to run tests.

Argument:
  [Optional] Dilla Design System name

Commands:
  run              Run tests for Design System as argument
  all              Run all tests for Dilla Design Systems defined as DILLA_DS_NAMES
  gen              Generate HTML result files for tests and examples under ./var/run
  gen_all          Generate HTML for Dilla Design Systems
  info | i         Get variables info used by this script

Options:
  -h --help        Display this help information

Usage:
  ${_ME} run
  ${_ME} run [_DS_NAME_]
  ${_ME} gen [_DS_NAME_]
HEREDOC
}

###############################################################################
# Specific script variables
###############################################################################

###############################################################################
# Script functions
###############################################################################

_run() {
  _init
  __test "${_ARGS}"
}

_all() {
  _init
  IFS=, read -ra values <<< "$DILLA_DS_NAMES"
  for _ds in "${values[@]}"; do
    if ! _blank "${_ds}"; then
      __test "${_ds}"
    fi
  done
}

_gen() {
  _init
  __gen "${DS}"
}

_gen_all() {
  _init
  IFS=, read -ra values <<< "$DILLA_DS_NAMES"
  for _ds in "${values[@]}"; do
    if ! _blank "${_ds}"; then
      __gen "${_ds}"
    fi
  done
}

###############################################################################
# Programs internal functions
###############################################################################


__test() {
  local DS=${1-}

  _cp_ds
  _log_debug "Run tests: test_${DS}..."

  DS=${DS} cargo test -p dilla-renderer --no-default-features -F test_ds --test tests_integrations -- --exact --nocapture || true
}

__test_docker() {
  local _action=${1:-"run"}
  docker pull "${DILLA_DOCKER_RUST}"

  mkdir -p "${_DILLA_ROOT_DIR}/.cargo"

  docker run --rm -t \
    -e CARGO_HOME=/app/.cargo \
    -e _DILLA_ROOT_DIR=/app/ \
    -v "${_DILLA_ROOT_DIR}/":/app -w /app \
    "${DILLA_DOCKER_RUST}" \
    ./scripts/test.sh "${_action}"
}

__gen() {
  local DS=${1-}

  if ! [[ -d "${DILLA_INPUT_DIR}/${DS}" ]]; then
    _log_error "DILLA_DS ${DS} input dir not found at ${DILLA_INPUT_DIR}/${DS}."
    _exit_1
  fi

  _cp_ds

  _log_info "Gen DS: ${DS} tests into ${DILLA_INPUT_DIR}/${DS}..."

  _ds_path="${DILLA_OUTPUT_DIR}/${DS}/"
  _origin_path="${DILLA_INPUT_DIR}/${DS}/"

  if ! [[ -d "${_ds_path}" ]]; then
    _log_error "Input dir not found at ${_ds_path}."
    _exit_1
  fi

  __gen_tests "${_ds_path}" "${_origin_path}"
  __gen_examples "${_ds_path}" "${_origin_path}"
}

__gen_tests() {
  local _ds_path=${1-}
  local _origin_path=${2-}

  for _file in "${_ds_path}tests/"*.json; do
    _full_name=$(basename "${_file%.*}")
    _output_file="${_ds_path}tests/${_full_name}.html"
    _cp_file="${_origin_path}tests/${_full_name}.html"

    component="${_full_name%%--*}"
    preview="${_full_name#*--}"

    __do_gen "${component}:${preview}" "${_file}" "${_output_file}"
    rm -f "${_cp_file}"
    cp "${_output_file}" "${_cp_file}"
  done

}
__gen_examples() {
  local _ds_path=${1-}
  local _origin_path=${2-}

  for _file in "${_ds_path}examples/"*.json; do
    _full_name=$(basename "${_file%.*}")
    _output_file="${_ds_path}examples/${_full_name}.html"
    _cp_file="${_origin_path}tests/${_full_name}.html"

    __do_gen "${_full_name}" "${_file}" "${_output_file}" "full"
    rm -f "${_cp_file}"
    cp "${_output_file}" "${_cp_file}"
  done
}

__do_gen() {
  local _name=${1-}
  local _test_payload=${2-}
  local _output_file=${3-}
  local _mode=${4-'_test'}

  if [ "${_name}" = "*" ]; then
    return
  fi

  _log_info "Generate HTML result as ${_mode} for ${_name}..."

  if ((_DRY_RUN)); then
    _log_notice "DS=${DS} cargo run -q --no-default-features -- render ${_test_payload} -m ${_mode} -w ${_output_file}"
  else
    DS=${DS} cargo run -q --no-default-features -- render "${_test_payload}" -m "${_mode}" -w "${_output_file}" || true
  fi
}

###############################################################################
# Programs functions used by _base.sh
###############################################################################

_local_check() {
  if [ "${DS}" == "test" ]; then
    _log_error "'DS' is set to 'test'"
    _error=1
  fi
}

# shellcheck source=/dev/null
if [ -f "${_DIR}/_base.sh" ]; then source "${_DIR}/_base.sh"; else
  echo -e "\e[31m[Error] Missing _base.sh file!\e[0m"
  exit 1
fi
