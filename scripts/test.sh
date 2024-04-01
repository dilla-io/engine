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
  int              Run Dilla internal tests, all or named as argument from _TESTS_FOLDER/tests.rs
  gen              Generate HTML result files for tests under ./var/run_ds_src/DS
  info | i         Get variables info used by this script

Options:
  -nb --no-build   Skip dilla build for test to speed up multiple tests run
  -h --help        Display this help information

Usage:
  ${_ME} run
  ${_ME} int --no-build
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

_int() {
  _init
  __test_int "${_ARGS}"
}

_gen() {
  _init
  __gen "${DS}"
}

###############################################################################
# Programs internal functions
###############################################################################

__test_int() {
  local _type=${1:-"all"}

  __dilla_test="${_TESTS_FOLDER}/tests.rs"
  if ! [ -f "${__dilla_test}" ]; then
    _log_error "Missing Dilla test file: ${__dilla_test}"
    _exit_1
  fi

  _log_debug "Run tests: test_${_type}..."

  DS=test cargo test $_QUIET --package dilla-renderer --test tests_core --no-fail-fast -- "test_${_type}" --exact --nocapture
}

__test() {
  local DS=${1-}

  __path="${DILLA_RUN_DS_FOLDER}/${DS}/tests"
  if ! [ -d "${__path}" ]; then
    _log_error "[SKIP] Missing test folder: ${__path}"
    return
  fi

  __path_co="${DILLA_RUN_DS_FOLDER}/${DS}/components"
  if ! [ -d "${__path_co}" ]; then
    _log_error "[SKIP] Missing test folder: ${__path_co}"
    return
  fi

  _cp_ds

  if ! ((_NO_BUILD)); then
    # shellcheck source=/dev/null
    source "${_DIR}"/build-cli.sh run
  else
    _log "Skip Rust build step for ${DS} tests"
  fi

  _log_debug "Run tests: test_${DS}..."
  DS=${DS} cargo test -p dilla-renderer --no-default-features --test tests_integrations -- --exact --nocapture || true
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

  _components_path="${DILLA_OUTPUT_DIR}/${DS}/components/"
  _tests_path="${DILLA_INPUT_DIR}/${DS}/tests/"

  if ! [[ -d "${_components_path}" ]]; then
    _log_error "Input dir not found at ${_components_path}."
    _exit_1
  fi

  _list_payload=('')
  for _file in "$_components_path"/*; do
    if [ -d "$_file" ]; then
      name=$(basename "${_file%.*}")
      _output_file=${_tests_path}${name}.html

      # Do we have a test payload?
      _test_payload=${_tests_path}${name}.json
      if [ ! -f "${_test_payload}" ]; then
        # If not a preview.json?
        _test_payload=${_components_path}${name}/preview.json
        if [ ! -f "${_test_payload}" ]; then
          # Skip gen for this component.
          _log_debug "[Skip] No payload found for: ${name}"
          continue;
        fi
      fi
      _list_payload+=("${name}")

      __do_gen "${name}" "${_test_payload}" "${_output_file}"
    fi
  done

  # Check if we have any other payload.
   _files="${_tests_path}*.json"
  for _file in ${_files}; do
    name=$(basename "${_file%.*}")
    _output_file=${_tests_path}${name}.html

    # Skip the common all lib test.
    if [ "${name}" = "_libraries" ]; then
      continue
    fi

    if ! _contains "${name}" "${_list_payload[@]}"; then
      __do_gen "${name}" "${_file}" "${_output_file}"
    fi
  done
}

__do_gen() {
  local _name=${1-}
  local _test_payload=${2-}
  local _output_file=${3-}

  if [ "${_name}" = "*" ]; then
    return
  fi

  _log_info "Generate HTML result as _test for ${name}..."

  if ((_DRY_RUN)); then
    _log_notice "DS=${DS} cargo run -q --no-default-features -- render ${_test_payload} -m _test -w ${_output_file}"
  else
    DS=${DS} cargo run -q --no-default-features -- render "${_test_payload}" -m "_test" -w "${_output_file}" || true
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
