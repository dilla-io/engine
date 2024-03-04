#!/bin/bash
#
# Dilla WASM builder from Design System with WASM-bindgen.
# shellcheck disable=SC2086,SC2209

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
  -tf              For test gen command, use _test_full instead of _test output
  -tp              For test gen command, path under /tests/, default /tests/component/
  -h --help        Display this help information

Usage:
  ${_ME} run
  ${_ME} int --no-build
  ${_ME} run [_DS_NAME_]
  ${_ME} gen [_DS_NAME_]
  ${_ME} gen [_DS_NAME_]
  ${_ME} gen [_DS_NAME_] -tf -tp component_with_lib
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

  _log_info "Run tests: test_${_type}..."

  DS=test cargo test $_QUIET --package dilla-renderer --test tests  --no-fail-fast -- "test_${_type}" --exact --nocapture
}

__test() {
  local DS=${1-}

  __path="${DILLA_RUN_DS_FOLDER}/${DS}/tests"

  if ! [ -d "${__path}" ]; then
    _log_error "[SKIP] Missing test folder: ${__path}"
    return
  fi

  _log_info "Run tests: test_${DS}...DO NOT INTERRUPT!"

  if [ -d "${_TESTS_FOLDER}/${DS}" ]; then
    rm -rf "${_TESTS_FOLDER}/${DS:?}"
  fi
  mkdir -p "${_TESTS_FOLDER}/${DS}"
  ln -s "${__path}" "${_TESTS_FOLDER}/${DS}"

  __dilla_test="${_TESTS_FOLDER}/tests.rs"
  if ! [ -f "${__dilla_test}" ]; then
    _log_error "Missing Dilla test file: ${__dilla_test}"
    _exit_1
  fi

  local _ds_test="${_TESTS_FOLDER}/${DS}/tests/tests.rs"
  if ! [ -f "${_ds_test}" ]; then
    _log_error "Missing DS test file: ${_ds_test}"
    _exit_1
  fi

  # Append our DS test function(s) to Dilla tests file.
  if ! [ -f "${__dilla_test}.bak" ]; then
    rm -f "${__dilla_test}.bak"
  fi

  cp "${__dilla_test}" "${__dilla_test}.bak"
  echo "// TMP test, will be removed at the end of tests, if not remove it!" >>"${__dilla_test}"
  cat "${_ds_test}" >>"${__dilla_test}"

  if ! ((_NO_BUILD)); then
    # shellcheck source=/dev/null
    source "${_DIR}"/build-cli.sh run
  else
    _log "Skip Rust build step for ${DS} tests"
  fi

  cargo test $_QUIET --package dilla-renderer --test tests -- "test_${DS}" --exact --nocapture || true
  # DS=${DS} cargo test $_QUIET --package dilla-renderer --test tests -- "test_${DS}" --exact --nocapture || true

  # _log "Tests cleanup..."
  rm -rf "${_TESTS_FOLDER}/${DS:?}"
  rm -f "${__dilla_test}"
  mv "${__dilla_test}.bak" "${__dilla_test}"
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

  _log_info "Gen DS: ${DS} tests into ${DILLA_INPUT_DIR}/${DS}..."

  _path="${DILLA_INPUT_DIR}/${DS}/tests/${_TEST_PATH}/"

  if ! [[ -d "${_path}" ]]; then
    _log_error "Input dir not found at ${_path}."
    _exit_1
  fi

  _files="${_path}*.json"
  for _file in ${_files}; do
    filename=$(basename "${_file%.*}")
    _output_file=${_path}${filename}.html
    _log_info "Generate HTML result as ${_TEST_OUTPUT} for ${filename}..."
    cargo run -q -- render "${_file}" -m "${_TEST_OUTPUT}" -w "${_output_file}"
    # DS=${DS} cargo run -q -- render "${_file}" -m "${_TEST_OUTPUT}" -w "${_output_file}"
  done
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
