#!/usr/bin/env bash
# shellcheck disable=SC2086,SC2216,SC2220

set -o nounset
set -o errexit
trap 'echo "Aborting due to errexit on line $LINENO. Exit code: $?" >&2' ERR
set -o errtrace
set -o pipefail
IFS=$'\n\t'

_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${_DIR}/../../" || exit

_print_help() {
  cat <<HEREDOC
Run benchmarks for Dilla WASM

Syntax: bench.sh <options>

Parameters:
  --node-only     Node perf tests only
  --hyper-only    Hyperfine tests only (Node with bootstrap)
  -t | --test     Smoke test, just run render() to check everything works.
  -d | --ds       Design system to use, default: bootstrap_5
  -s | --size     Can be any of xs,sm,md,l,xl, default: sm.
  -r | --runs     Number of bench runs, default: 20
  -w | --warmup   Number of bench warmup, default: 2

USage example:
  bench.sh -r 100 -w 2 --node-only
HEREDOC
}

# Parse Options ###############################################################

_PRINT_HELP=0
_TEST_RUN=0
_RUN_ONLY_NODE=0
_RUN_ONLY_HYPERFINE=0

_DS="bootstrap_5"
_PAYLOAD_SIZE="sm"
_RUNS=20
_WARMUP=2

# Initialize additional expected option variables.
_CMD="run"
# _CMD="print_help"
# _CMD=()

__get_option_value() {
  local __arg="${1:-}"
  local __val="${2:-}"

  if [[ -n "${__val:-}" ]] && [[ ! "${__val:-}" =~ ^- ]]; then
    printf "%s\\n" "${__val}"
  else
    _exit_1 printf "%s requires a valid argument.\\n" "${__arg}"
  fi
}

while ((${#})); do
  __arg="${1:-}"
  __val="${2:-}"

  case "${__arg}" in
  -h | --help)
    _PRINT_HELP=1
    ;;
  --node-only)
    _RUN_ONLY_NODE=1
    ;;
  --hyper-only)
    _RUN_ONLY_HYPERFINE=1
    ;;
  -t | --test)
    _CMD="smoke"
    ;;
  -d | --ds)
    _DS="$(__get_option_value "${__arg}" "${__val:-}")"
    shift
    ;;
  -s | --size)
    _PAYLOAD_SIZE="$(__get_option_value "${__arg}" "${__val:-}")"
    shift
    ;;
  -r | --runs)
    _RUNS="$(__get_option_value "${__arg}" "${__val:-}")"
    shift
    ;;
  -w | --warmup)
    _WARMUP="$(__get_option_value "${__arg}" "${__val:-}")"
    shift
    ;;
  --endopts)
    # Terminate option parsing.
    break
    ;;
  -*)
    _exit_1 printf "Unexpected option: %s\\n" "${__arg}"
    ;;
  *)
    # [[ADAPT]]
    # _CMD=${__option}
    _CMD+=("$1")
    # shift
    ;;
  esac
  shift
done

if [ -z "${_CMD:-}" ]; then
  _print_help
  exit 0
fi

__payload_file="$_DIR/../../tmp/payload_${_PAYLOAD_SIZE}.json"
__target="$_DIR/../../dist/${_DS}"

###############################################################################
# Program Functions
###############################################################################

_copy_payload() {
  if [ ! -f "${__payload_file}" ]; then
    echo -e "[Error] Missing payload: ${__payload_file}"
    exit 1
  fi

  if [ ! -d "${__target}/component/payload/" ]; then
    echo -e "[Error] Missing folder: ${__target}/component/payload/"
    exit 1
  fi
  if [ ! -d "${__target}/bindgen/payload/" ]; then
    echo -e "[Error] Missing folder: ${__target}/component/payload/"
    exit 1
  fi
  if [ ! -d "${__target}/extism/payload/" ]; then
    echo -e "[Error] Missing folder: ${__target}/component/payload/"
    exit 1
  fi

  cp "${__payload_file}" ${__target}/component/payload/index.json
  cp "${__payload_file}" ${__target}/bindgen/payload/index.json
  cp "${__payload_file}" ${__target}/extism/payload/index.json
}

_smoke() {
  __payload_file="$_DIR/../../tmp/payload_test.json"
  _copy_payload
  echo -e "[TEST] Rust bin..."
  dist/bin/${_DS} render -m json -r dist/${_DS}/component/payload/index.json
  echo -e "[TEST] wasmtime..."
  wasmtime --dir=. dist/${_DS}/component/wasm/${_DS}.wasm render dist/${_DS}/component/payload/index.json
  echo -e "[TEST] Component Node..."
  node --no-warnings dist/${_DS}/component/node/render.mjs
  echo -e "[TEST] Component Node WASI..."
  node --no-warnings dist/${_DS}/component/node/wasi.mjs
  echo -e "[TEST] Bindgen Node..."
  node --no-warnings dist/${_DS}/bindgen/node/render.mjs
  echo -e "[TEST] Extism Node..."
  node --no-warnings dist/${_DS}/extism/node/render.mjs
}

_run() {
  _copy_payload
  if ((!_RUN_ONLY_NODE)); then
    _run_hyperfine
  fi
  if ((!_RUN_ONLY_HYPERFINE)); then
    _run_node
  fi
}

_run_hyperfine() {
      # -n "Bin" \
      # "dist/bin/${_DS} render -m json -r dist/${_DS}/component/payload/index.json" \
  hyperfine --runs $_RUNS --warmup $_WARMUP \
    -n "Wasmtime" \
    -n "Component wasi" \
    -n "Wasm-Bindgen" \
    -n "Extism" \
    -n "Component jco" \
    --export-markdown "${_DIR}/report/bench_${_DS}_${_RUNS}.md" \
    --export-csv "${_DIR}/report/bench_${_DS}_${_RUNS}.csv" \
    "wasmtime --dir=. dist/${_DS}/component/wasm/${_DS}.wasm render dist/${_DS}/component/payload/index.json" \
    "node --no-warnings dist/${_DS}/component/node/wasi.mjs render" \
    "node --no-warnings dist/${_DS}/bindgen/node/render.mjs" \
    "node --no-warnings dist/${_DS}/extism/node/render.mjs" \
    "node --no-warnings dist/${_DS}/component/node/render.mjs"
}

_run_node() {
  echo -e "Benchmark: Bindgen Node"
  _bench_bg_node=$(node --no-warnings dist/${_DS}/bindgen/node/bench.mjs $_RUNS $_WARMUP)
  echo -e "  Time\t\t\t$_bench_bg_node"

  echo -e "Benchmark: Component Node WASI"
  _bench_co_node_wasi=$(node --no-warnings dist/${_DS}/component/node/bench_wasi.mjs $_RUNS $_WARMUP)
  _bench_co_node_wasi=$(echo "${_bench_co_node_wasi}" | sed '/^[[:space:]]*$/d')
  echo -e "  Time\t\t\t${_bench_co_node_wasi}"

  echo -e "Benchmark: Extism Node"
  _bench_ex_node=$(node --no-warnings dist/${_DS}/extism/node/bench.mjs $_RUNS $_WARMUP)
  echo -e "  Time\t\t\t$_bench_ex_node"

  echo -e "Benchmark: Component Node"
  _bench_co_node=$(node --no-warnings dist/${_DS}/component/node/bench.mjs $_RUNS $_WARMUP)
  echo -e "  Time\t\t\t$_bench_co_node"

  _size=$(stat -c '%s' "${__payload_file}" | numfmt --to=iec)
  _lines=$(wc -l <"${__payload_file}")
  echo -e "\nPayload size: ${_size} Lines: ${_lines}"
}

###############################################################################
# Main
###############################################################################

# _main()
#
# Usage:
#   _main [<options>] [<arguments>]
#
# Description:
#   Entry point for the program, handling basic option parsing and dispatching.
_main() {
  if ((_PRINT_HELP)); then
    _print_help
  else
    # Run command if exist.
    __call="_${_CMD}"
    if [[ "$(type -t "${__call}")" == 'function' ]]; then
      $__call
    else
      echo -e "[Error] Unknown command: ${_CMD}"
    fi
  fi
}

_main "$@"
