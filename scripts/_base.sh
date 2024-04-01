#!/usr/bin/env bash
#
# Helper.
#
# Bash Boilerplate: https://github.com/alphabetum/bash-boilerplate
#
# Copyright (c) 2015 William Melody • hi@williammelody.com
# shellcheck disable=SC2034

if [ -z "${_INIT_DONE-}" ] && [ -f "./_init.sh" ]; then
  # shellcheck source=/dev/null
  source ./_init.sh
fi

###############################################################################
# Utilities
###############################################################################

# _exit_1()
#
# Usage:
#   _exit_1 <command>
#
# Description:
#   Exit with status 1 after executing the specified command with output
#   redirected to standard error. The command is expected to print a message
#   and should typically be either `echo`, `printf`, or `cat`.
_exit_1() {
  {
    # Prefix die message with "cross mark (U+274C)", often displayed as a red x.
    printf "%s " "$(tput setaf 1)❌$(tput sgr0)"
    "${@}"
  } 1>&2
  exit 1
}

# die()
#
# Usage:
#   die "Error message. Variable: $0"
#
# Exit with an error and print the specified message.
#
# This is a shortcut for the _die() function that simply echos the message.
die() {
  _exit_1 echo "${@}"
}

###############################################################################
# Helpers
###############################################################################

# _function_exists()
#
# Usage:
#   _function_exists <name>
#
# Exit / Error Status:
#   0 (success, true) If function with <name> is defined in the current
#                     environment.
#   1 (error,  false) If not.
#
# Other implementations, some with better performance:
# http://stackoverflow.com/q/85880
_function_exists() {
  [ "$(type -t "${1}")" == 'function' ]
}

# _command_exists()
#
# Usage:
#   _command_exists <name>
#
# Exit / Error Status:
#   0 (success, true) If a command with <name> is defined in the current
#                     environment.
#   1 (error,  false) If not.
#
# Information on why `hash` is used here:
# http://stackoverflow.com/a/677212
_command_exists() {
  hash "${1}" 2>/dev/null
}

# _blank()
#
# Usage:
#   _blank <argument>
#
# Exit / Error Status:
#   0 (success, true)  If <argument> is not present or null.
#   1 (error,  false)  If <argument> is present and not null.
_blank() {
  [[ -z ${1-} ]]
}

# _present()
#
# Usage:
#   _present <argument>
#
# Exit / Error Status:
#   0 (success, true)  If <argument> is present and not null.
#   1 (error,  false)  If <argument> is not present or null.
_present() {
  [[ -n ${1-} ]]
}

# _contains()
#
# Usage:
#   _contains <query> <list-item>...
#
# Exit / Error Status:
#   0 (success, true)  If the item is included in the list.
#   1 (error,  false)  If not.
#
# Examples:
#   _contains "${_query}" "${_list[@]}"
_contains() {
  local _query="${1-}"
  shift

  if [[ -z ${_query} ]] ||
    [[ -z ${*-} ]]; then
    return 1
  fi

  for __element in "${@}"; do
    [[ ${__element} == "${_query}" ]] && return 0
  done

  return 1
}

# _download_from()
#
# Usage:
#   _download_from <url> [<outfile>]
#
# Description:
#   Download the file at <url> and print to standard output or <outfile>, if
#   present. Uses `curl` if available, falling back to `wget`. Messages from
#   `curl` and `wget` are suppressed.
#
# Exit / Error Status:
#   0 (success, true)  If the download is successful.
#   1 (error,  false)  If there was an error.
#
# Examples:
#   # Download and stream to standard output.
#   _download_from "https://example.com" | less
#
#   # Download to outfile with error handling.
#   if ! _download_from "https://example.com/example.pdf" /path/to/example.pdf
#   then
#     printf "Download error.\\n"
#     exit 1
#   fi
_download_from() {
  local _downloaded=0
  local _target_path="${2-}"
  local _timeout=15
  local _url="${1-}"

  if [[ -z ${_url} ]] || [[ ! ${_url} =~ ^https\:|^http\:|^file\:|^ftp\:|^sftp\: ]]; then
    _log_error "File url is not valid: ${_url}"
    return 1
  fi

  if [[ -n ${_target_path} ]]; then
    if hash "curl" 2>/dev/null; then
      curl \
        --location \
        --connect-timeout "${_timeout}" \
        "${_url}" \
        --output "${_target_path}" && _downloaded=1
    elif hash "wget" 2>/dev/null; then
      wget \
        --connect-timeout="${_timeout}" \
        --dns-timeout="${_timeout}" \
        -O "${_target_path}" \
        "${_url}" \
        2>/dev/null && _downloaded=1
    fi
  else
    if hash "curl" 2>/dev/null; then
      curl \
        --location \
        --connect-timeout "${_timeout}" \
        "${_url}" && _downloaded=1
    elif hash "wget" 2>/dev/null; then
      wget \
        --connect-timeout="${_timeout}" \
        --dns-timeout="${_timeout}" \
        -O - \
        "${_url}" \
        2>/dev/null && _downloaded=1
    fi
  fi

  if ! ((_downloaded)); then
    return 1
  fi
}

# _join()
#
# Usage:
#   _join <separator> <array>
#
# Examples:
#   _join , a "b c" d     => a,b c,d
#   _join / var local tmp => var/local/tmp
#   _join , "${FOO[@]}"   => a,b,c
#
# More Information:
#   http://stackoverflow.com/a/17841619
_join() {
  local _delimiter="${1}"
  shift
  printf "%s" "${1}"
  shift
  printf "%s" "${@/#/${_delimiter}}" | tr -d '[:space:]'
}

# _prompt_yn()
#
# Usage:
#   _prompt_yn <Text to display>
_prompt_yn() {
  printf '%s\n' "$1"
  read -r -p "Are you sure (y/n)? " answer
  case ${answer:0:1} in
  y | Y) ;;
  *)
    printf 'Canceled, bye!\n'
    exit 1
    ;;
  esac
}

# _spinner()
#
# Usage:
#   _spinner <pid>
#
# Description:
#   Display an ascii spinner while <pid> is running.
#
# Example Usage:
#   ```
#   _spinner_example() {
#     printf "Working..."
#     (sleep 1) &
#     _spinner $!
#     printf "Done!\n"
#   }
#   (_spinner_example)
#   ```
#
# More Information:
#   http://fitnr.com/showing-a-bash-spinner.html
_spinner() {
  local _pid="${1-}"
  local _delay=0.75
  # shellcheck disable=SC1003
  local _spin_string='|/-\'

  if [[ -z ${_pid} ]]; then
    printf 'Usage: _spinner <pid>\n'
    return 1
  fi

  while ps a | awk '{print $1}' | grep -q "${_pid}"; do
    local _temp="${_spin_string#?}"
    printf " [%c]  " "${_spin_string}"
    # shellcheck disable=SC2295
    _spin_string="${_temp}${_spin_string%${_temp}}"
    sleep ${_delay}
    printf '\b\b\b\b\b\b'
  done
  printf '    \b\b\b\b'
}

# _size()
#
# Usage:
#   _size <path>
#
# Description:
#   Display human readable size from <path>.
#
# Example Usage:
#   ```
#   _size ./my/file.txt
#   ```
_size() {
  local _path="${1-}"
  stat -c '%s' "${_path}" | numfmt --to=iec
}

# _send_remote()
#
# Usage:
#   _send_remote <local_path> <remote> <remote_path>
#
# Description:
#   Create and extract an archive from local <local_path> to distant server <remote><remote_path>.
#
# Example Usage:
#   ```
#   _send_remote local/dist bob@my_server.net /some/remote/path
#   ```
_send_remote() {
  local _src_dir="${1-}"
  local _remote="${2-}"
  local _remote_dir="${3-}"
  local _archive="${4:-"tmp"}.tgz"

  local _error=

  if ! command -v ssh &>/dev/null; then
    _log_error "'ssh' could not be found, please install"
    _error=1
  fi

  if ! command -v scp &>/dev/null; then
    _log_error "'scp' could not be found, please install"
    _error=1
  fi

  if ! [[ -d ${_src_dir} ]]; then
    _log_error "Missing source folder '${_src_dir}'"
    _error=1
  fi

  if _blank "${_remote-}"; then
    _log_error "Missing second argument remote address"
    _error=1
  fi

  if _blank "${_remote_dir-}"; then
    _log_error "Missing third argument remote path"
    _error=1
  fi

  if ((_error)); then
    _exit_1
  fi

  if [[ -f ${_archive} ]]; then
    rm -f "${_archive}"
  fi

  _log_info "Send ${_src_dir} to ${_remote_dir}"

  if ((_DRY_RUN)); then
    return
  fi

  _log_notice "Create archive ${_archive} from ${_src_dir}"

  if ! _blank "${_QUIET}"; then
    tar -czf "${_archive}" --exclude "node_modules" --exclude "package*.json" -C "${_src_dir}" . >/dev/null 2>&1
  else
    tar -czvf "${_archive}" --exclude "node_modules" --exclude "package*.json" -C "${_src_dir}" .
  fi

  __size=$(_size "${_archive}")
  _log_notice "Created archive ${_archive} ${__size} successfully!"

  _log_notice "Upload archive to ${_remote_dir}..."
  ssh -4 "${_remote}" rm -rf "${_remote_dir}/*"
  scp -4 "${_archive}" "${_remote}:/tmp"
  _log_notice "Archive uploaded successfully!"

  _log_notice "Extract archive on remote..."
  ssh -4 "${_remote}" mkdir -p "${_remote_dir}"
  ssh -4 "${_remote}" tar -xzf "/tmp/${_archive}" -C "${_remote_dir}"
  _log_notice "Archive extracted successfully!"

  _log_notice "Cleanup.."
  if _blank "${_QUIET}"; then
    ssh -4 "${_remote}" ls -lAh "${_remote_dir}"
  fi
  ssh -4 "${_remote}" rm -rf "/tmp/${_archive}"
  rm -f "${_archive}"
  _log_success "Archive ${_archive} uploaded and extracted to ${_remote_dir}!"
}

# _sed_vars()
#
# Usage:
#   _sed_vars <target>
#
# Description:
#   Replace DILLA vars in a file.
#
# Example Usage:
#   ```
#   _sed_vars local/target bindgen
#   ```
_sed_vars() {
  local _target="${1-}"
  local _type="${2-:}"

  if [[ ! -f "${_target}" ]]; then
    _log_error "Missing target ${_target}"
    _exit_1
  fi

  if [ -n "${_type}" ]; then
    sed -i "s|#TYPE#|${_type:-""}|" "${_target}"
  fi

  sed -i "s|#DS#|${DS:-"__n_a__"}|" "${_target}"
  sed -i "s|#VERSION#|${VERSION:-"__n_a__"}|" "${_target}"
  sed -i "s|#GITLAB_TOKEN#|${GITLAB_TOKEN:-"__n_a__"}|" "${_target}"
  sed -i "s|#DILLA_PROJECT_ID#|${DILLA_PROJECT_ID:-"__n_a__"}|" "${_target}"

  sed -i "s|#DILLA_DATA_JSON_URL#|${DILLA_DATA_JSON_URL:-"__n_a__"}|" "${_target}"
  sed -i "s|#DILLA_DIST_LOCAL_JS_HELPER#|${DILLA_DIST_LOCAL_JS_HELPER:-"__n_a__"}|" "${_target}"
  sed -i "s|#DILLA_JS_HELPER_URL#|${DILLA_JS_HELPER_URL:-"__n_a__"}|" "${_target}"

  sed -i "s|#DILLA_DIST_LOCAL_WASM_COMPONENT_FOLDER#|${DILLA_DIST_LOCAL_WASM_COMPONENT_FOLDER:-"__n_a__"}|" "${_target}"

  sed -i "s|#_JS_BINDGEN_URL#|${_JS_BINDGEN_URL:-"__n_a__"}|" "${_target}"
}

# Set json tests and index files if available.
# @todo check the python generate.

# _set_json_data()
#
# Usage:
#   _set_json_data <json_file> <dest> <type>
#
# Description:
#   Copy or create a json file for packages (test.json or index.json)
#
# Example Usage:
#   ```
#   _set_json_data "some_path/test.json" "dist/bindgen/swing_1/test.json" "component"
#   ```
_set_json_data() {
  local _json_file=${1-}
  local _dest_name=${2-}
  local _type=${3-""}

  if [ -f "${_DILLA_ROOT_DIR}/${_json_file}" ]; then
    cp "${_DILLA_ROOT_DIR}/${_json_file}" "${_dest_name}"
  else
    touch "${_dest_name}"
    echo '[{"@element": "h2", "@content": "Hello from Dilla '"${DS:-"__n_a__"}"' '"${_type}"' v'"${VERSION:-"__n_a__"}"'!"}, "It'\''s working!"]' > "${_dest_name}"
  fi
}

# _post_build()
#
# Usage:
#   _post_build <source> <dest> [payload_type]
#
# Description:
#   Copy and replace template files
#
# Example Usage:
#   ```
#   _post_build "tpl_path" "target_path"
#   _post_build "tpl_path" "target_path" "type:bindgen" "payload_set"
#   ```
_post_build() {
  local _tpl=${1:-}
  local _target=${2:-}
  local _type=${3:-}
  local _payload=${4:-}

  if [ ! -d "${_tpl}" ]; then
    _log_warn_light "Unable to find tpl folder: ${_tpl}!"
    return
  fi

  _log_notice "Create packaging files from ${_tpl} to ${_target}..."
  mkdir -p "${_target}"

  find "${_tpl}" -maxdepth 1 -type f -print0 | while IFS= read -r -d '' filename; do
    _file=${filename##*/}
    cp "${filename}" "${_target}/"
    _sed_vars "${_target}/${_file}" "${_type}"
  done

  if ! _blank "${_payload}"; then
    _log_debug "Set payload file(s) in ${_DILLA_DS_TARGET}/payload"
    __prepare_dir "${_DILLA_DS_TARGET}/payload"
    # _set_json_data "${DILLA_DIST_WASM_BG_PLG_FOLDER}/${DS}/${DILLA_DATA_FILE_SRC_TESTS}" "${_target}/payload/test.json" "${_type}"
    _set_json_data "" "${_DILLA_DS_TARGET}/payload/index.json" "${_type}"
  fi

  if [ ! -d "${_target}/node_modules" ] && [ -f "${_target}/package.json" ]; then
    _log_debug "npm install -s ${_target}"
    cd "${_target}" && npm install -s
  elif [ -d "${_target}/node_modules" ] && [ -f "${_target}/package.json" ]; then
    _log_debug "npm update -s ${_target}"
    cd "${_target}" && npm update -s
  fi
}

_wasm_opt() {
  local _wasm=${1-}

  _log_debug "Run wasm-opt on ${_wasm} with '-Os --low-memory-unused --enable-bulk-memory'..."

  __size=$(_size "${_wasm}")
  wasm-opt -Os --low-memory-unused --enable-bulk-memory "${_wasm}" -o "${_wasm}"
  __size_after=$(_size "${_wasm}")

  _log_notice "WASM file successfully optimized! ${__size} > ${__size_after}"
}

###############################################################################
# Logging with colors
###############################################################################

__red=$'\e[1;31m'
__bck_red=$'\e[0;41m'
__grn=$'\e[1;32m'
__grn_light=$'\e[0;32m'
__bck_grn=$'\e[0;42m'
__yel=$'\e[0;33m'
__yel_light=$'\e[38;5;223m'
__blu=$'\e[1;34m'
__blu_light=$'\e[38;5;159m'
__dim=$'\e[0;90m'
__end=$'\e[0m'

# _log()
#
# Description:
#   Log the given message at the given level
#
# Usage:
#   _log <level> <message> <color> <timestamp>
#
# Description:
#   Display a message with a custom level and color.
#   All logs are written to stderr with a timestamp.
#
# Examples:
#   _log INFO "This is a message" "\e[0;34m"
#
# More Information (inspired from):
#   https://github.com/gruntwork-io/bash-commons/blob/master/modules/bash-commons/src/log.sh

function _log() {
  local -r message="${1-}"
  local -r level="${2:-notice}"
  local -r color="${3:-$__dim}"
  local -r txt_color="${4:-$__dim}"
  local -r timestamp=$(date +"%Y-%m-%d %H:%M:%S")
  if _present "${5-}"; then
    echo >&2 -e "[${timestamp}] ${color}[${level}]\e[0m ${txt_color}${message}\e[0m"
  else
    echo >&2 -e "${color}[${level}]\e[0m ${txt_color}${message}\e[0m"
  fi
}

function _log_info() {
  local -r message="$1"
  local -r show_time="${2-}"
  _log "$message" "Info" "$__blu" "$__blu" "$show_time"
}

function _log_error() {
  local -r message="$1"
  local -r show_time="${2-}"
  _log "$message" "Error" "$__bck_red" "$__red" "$show_time"
}

function _log_success() {
  local -r message="$1"
  local -r show_time="${2-}"
  _log "$message" "OK" "$__bck_grn" "$__grn" "$show_time"
}

function _log_debug() {
  if [ "${_DEBUG}" = "0" ]; then
    return
  fi
  local -r message="$1"
  local -r show_time="${2-}"
  _log "$message" "Debug" "$__dim" "$__dim" "$show_time"
}

function _log_notice() {
  if ! _blank "${_QUIET}" && [ "${_DEBUG}" = "0" ]; then
    return
  fi
  local -r message="$1"
  local -r show_time="${2-}"
  _log "$message" "notice" "$__blu_light" "$__blu_light" "$show_time"
}

function _log_warn_light() {
  if ! _blank "${_QUIET}" && [ "${_DEBUG}" = "0" ]; then
    return
  fi
  local -r message="$1"
  local -r show_time="${2-}"
  _log "$message" "warn" "$__yel_light" "$__yel_light" "$show_time"
}

function _log_warn() {
  if ! _blank "${_QUIET}" && [ "${_DEBUG}" = "0" ]; then
    return
  fi
  local -r message="$1"
  local -r show_time="${2-}"
  _log "$message" "warn" "$__yel" "$__yel" "$show_time"
}

function _log_success_light() {
  if ! _blank "${_QUIET}" && [ "${_DEBUG}" = "0" ]; then
    return
  fi
  local -r message="$1"
  local -r show_time="${2-}"
  _log "$message" "ok" "$__grn_light" "$__grn_light" "$show_time"
}

function _test_log() {
  _log_info "This is info"
  _log_error "This is error"
  _log_success "This is success"
  _log_notice "This is notice"
  _log_warn_light "This is warn light"
  _log_warn "This is warn"
  _log_success_light "This is success light"
  _log_debug "This is debug"

}

# Parse Options ###############################################################

# Initialize program option variables.
_DRY_RUN=0
_NO_BUILD=0
_NO_OPTIMIZATION=
_FEATURES=
_PRINT_HELP=0
_REPO=
_SKIP_DOCKER_PULL=0
_VERBOSE=0
_QUIET="--quiet"
_DEBUG=0

_IS_DOCKER=

# __get_option_value()
#
# Usage:
#   __get_option_value <option> <value>
#
# Description:
#  Given a flag (e.g., -e | --example) return the value or exit 1 if value
#  is blank or appears to be another option.
__get_option_value() {
  local __arg="${1-}"
  local __val="${2-}"

  if [[ -n ${__val-} ]] && [[ ! ${__val-} =~ ^- ]]; then
    printf '%s\n' "${__val}"
  else
    _exit_1 printf '%s requires a valid argument.\n' "${__arg}"
  fi
}

# Initialize additional expected option variables.
_CMD=()

while ((${#})); do
  __arg="${1-}"
  __val="${2-}"

  case "${__arg}" in
  -h | --help)
    _PRINT_HELP=1
    ;;
  --debug)
    _DEBUG=1
    ;;
  --dry | --dry-run)
    _DRY_RUN=1
    ;;
  -v | --verbose)
    _QUIET=""
    ;;
  -nb | --no-build)
    _NO_BUILD=1
    ;;
  -no | --no-optim)
    _NO_OPTIMIZATION=1
    ;;
  --is-docker)
    _IS_DOCKER=1
    ;;
  --version)
    VERSION="$(__get_option_value "${__arg}" "${__val-}")"
    shift
    ;;
  --skip-pull)
    _SKIP_DOCKER_PULL=1
    ;;
  --skip-check)
    _SKIP_CHECK_SCHEMAS=1
    ;;
  -r | --repository)
    _REPO="$(__get_option_value "${__arg}" "${__val-}")"
    shift
    ;;
  -f | --features)
    _FEATURES="-F $(__get_option_value "${__arg}" "${__val-}")"
    shift
    ;;
  --endopts)
    # Terminate option parsing.
    break
    ;;
  -*)
    _exit_1 printf 'Unexpected option: %s\n' "${__arg}"
    ;;
  *)
    _CMD+=("$1")
    ;;
  esac
  shift
done

if _blank "${_CMD-}"; then
  if _function_exists _print_help; then
    _print_help
  else
    _log_info "No help provided for this script."
  fi
  exit 0
fi

# shellcheck disable=SC2124
_ARGS=${_CMD[@]:1}

if _present "${_ARGS}"; then
  DS="${_ARGS}"
fi

###############################################################################
# PROGRAMS helpers for Dilla
###############################################################################

__prepare_dir() {
  local _dir=${1:-}
  if [ -d "${_dir}" ]; then
    rm -rf "${_dir}"
  fi
  mkdir -p "${_dir}"
}

_cp_ds() {

  if [ -f "${DILLA_LIB_FOLDER}/src/build/ds.rs" ]; then
    rm -f "${DILLA_LIB_FOLDER}/src/build/ds.rs"
  fi

  if ! [ -f "${DILLA_RUN_DS_FOLDER}/${DS}/ds.rs" ] && [ "${DS}" != "test" ]; then
    _log_error "Missing design system in ${DILLA_RUN_DS_FOLDER}/${DS}/ds.rs"
    _exit_1
  fi

  if [ "${DS}" == "test" ]; then
    _log_notice "Copy ds.rs from ${DILLA_LIB_FOLDER}/src/build/test.rs"
    cp "${DILLA_LIB_FOLDER}/src/build/test.rs" "${DILLA_LIB_FOLDER}/src/build/ds.rs"
  elif [ -f "${DILLA_RUN_DS_FOLDER}/${DS}/ds.rs" ]; then
    _log_notice "Copy ds.rs from ${DILLA_RUN_DS_FOLDER}/${DS}/ds.rs"
    cp "${DILLA_RUN_DS_FOLDER}/${DS}/ds.rs" "${DILLA_LIB_FOLDER}/src/build/ds.rs"
  fi
}

_docker_run_build() {
  local __type=${1:-"bg"}
  local __opt='--is-docker'

  _prepare_docker
  _valid_ds

  if ((_DRY_RUN)); then
    echo "./scripts/build-${__type}.sh run ${DS} ${__opt}"
    exit 0
  fi
  _log_info "Execute WASM build scripts from docker..."
  _docker_run_cmd "scripts/build-${__type}.sh run ${DS} ${__opt}"
}

_docker_build_all() {
  local __type=${1:-"bg"}
  local __opt=''
  _prepare_docker

  if ((_DRY_RUN)); then
    echo "./scripts/build-${__type}.sh run all ${__opt}"
    exit 0
  fi
  _log_info "Execute WASM build scripts from docker..."
  _docker_run_cmd "scripts/build-${__type}.sh all ${__opt}"
}

_prepare_docker() {

  if ((_SKIP_DOCKER_PULL)); then
    _log_warn_light "[SKIP] Docker pull!"
  else
    docker pull --quiet "${DILLA_DOCKER_RUST}"
  fi

  mkdir -p "${_DILLA_ROOT_DIR}/.cargo"

  __opt=''
  if ((_NO_BUILD)); then
    __opt+=' --no-build'
  fi
  if ((_NO_OPTIMIZATION)); then
    __opt+=' --no-optim'
  fi
  if ((_FEATURES)); then
    __opt+=" $_FEATURES"
  fi
  if ((_SKIP_DOCKER_PULL)); then
    __opt+=" --skip-pull"
  fi
  if _blank "${_QUIET}"; then
    __opt+=" --verbose"
  fi
  if _blank "${_DRY_RUN}"; then
    __opt+=" --dry-run"
  fi
  if _blank "${_IS_DOCKER}"; then
    __opt+=" --is-docker"
  fi
}

_docker_run_cmd() {
  local _cmd=${1:-"echo 'No commands for Docker!'"}
  _log_notice "Docker run: ${_cmd} on ${DILLA_DOCKER_RUST}"
  docker run --rm -t \
    -u "$(id -u):$(id -g)" \
    -e CARGO_HOME=/app/.cargo \
    -v "${_DILLA_ROOT_DIR}/":/app -w /app \
    "${DILLA_DOCKER_RUST}" bash -c "${_cmd}"
}

_valid_ds() {
  local _valid=
  IFS=, read -ra values <<<"$DILLA_DS_NAMES"
  for _ds in "${values[@]}"; do
    if [ "${_ds}" == "${DS}" ]; then
      _valid=1
    fi
  done

  if _blank "${_valid-}"; then
    _log_error "Invalid DS name: ${DS}"
    _log_info "Available: $DILLA_DS_NAMES"
    _log_info "Note: Names list is set in your '.env'"
    _exit_1
  fi
}

_ds_req() {
  if _blank "${DS-}"; then
    _log_error "Missing DS NAME! Set with DS=swing_1 or as first argument of this script."
    _exit_1
  fi

  if [ "${DS}" == "__NAME_TO_SET__" ]; then
    _log_error "Missing DS NAME! Set with DS=swing_1 or as first argument of this script."
    _exit_1
  fi
}

_ds_not_test() {
  if _blank "${DS-}"; then
    _log_error "Missing DS NAME! Set with DS=swing_1 or as first argument of this script."
    _exit_1
  fi

  if [ "${DS}" == "test" ]; then
    _log_error "Missing DS NAME (current is test)! Set with DS=swing_1 or as first argument of this script."
    _exit_1
  fi
}

_token_req() {
  if _blank "${GITLAB_TOKEN-}"; then
    _log_error "Missing GITLAB_TOKEN, check .env"
    _exit_1
  fi
}

_set() {
  if _blank "${DS-}" && _present "${CI_PROJECT_NAME-}"; then
    DS="${CI_PROJECT_NAME}"
  fi

  if _function_exists _local_init; then
    _local_init
  fi
}

_init() {
  _set
  _check
}

_check() {
  _error=0

  if _function_exists _local_check; then
    _local_check
  fi

  if [ $_error == 1 ]; then
    _exit_1
  fi
}

_info() {
  _set

  _global_info

  if _function_exists _local_info; then
    _local_info
  fi
}

_global_info() {
  _set
  _log_info "Global variables:"
  echo -e " DS:\t\t\t\t${DS-}"
  echo -e " VERSION:\t\t\t${VERSION:-"-"}"
  echo -e " DILLA_PROJECT_ID:\t\t${DILLA_PROJECT_ID:-"-"}"
  # shellcheck disable=SC2145
  echo -e " DILLA_DS_NAMES:\t\t${DILLA_DS_NAMES[@]}"
  echo -e " --"
  echo -e " DILLA_DIST_FOLDER:\t\t${DILLA_DIST_FOLDER:-"-"}"
  echo -e " DILLA_RUN_DS_FOLDER:\t\t${DILLA_RUN_DS_FOLDER:-"-"}"
  echo -e " --"
  echo -e " DILLA_LIB_FOLDER:\t\t${DILLA_LIB_FOLDER:-"-"}"
  echo -e " _TESTS_FOLDER:\t\t\t${_TESTS_FOLDER:-"-"}"
  echo -e " --"
  echo -e " DILLA_DOCKER_RUST:\t\t${DILLA_DOCKER_RUST:-"-"}"
  echo -e " DILLA_DOCKER_SCHEMAS:\t\t${DILLA_DOCKER_SCHEMAS:-"-"}"
  echo -e " DILLA_DOCKER_PREBUILDER:\t${DILLA_DOCKER_PREBUILDER:-"-"}"
  echo -e " --"
  echo -e " DILLA_DATA_URL:\t\t${DILLA_DATA_URL:-"-"}"
  echo -e " DILLA_WASM_BINDGEN_URL:\t${DILLA_WASM_BINDGEN_URL:-"-"}"
  echo -e " --"
  if _present "${CI-}"; then
    echo -e " CI:\t\t\t\t${CI:-"YES"}"
    echo -e " CI_PROJECT_NAME:\t\t${CI_PROJECT_NAME:-"-"}"
    echo -e " CI_PROJECT_ID:\t\t${CI_PROJECT_ID:-"-"}"
    echo -e " CI_COMMIT_TAG:\t\t\t${CI_COMMIT_TAG:-"-"}"
    echo -e " CI_PROJECT_DIR:\t\t${CI_PROJECT_DIR:-"-"}"
    echo -e " CI_PAGES_URL:\t\t\t${CI_PAGES_URL:-"-"}"
  else
    echo -e " CI:\t\t\t\t${CI:-"NO"}"
  fi
  echo -e " CI_API_V4_URL:\t\t\t${CI_API_V4_URL:-"-"}"
  echo -e " --"
  echo -e " _DILLA_ROOT_DIR:\t\t${_DILLA_ROOT_DIR:-"-"}"
  echo -e " _DIR:\t\t\t\t${_DIR:-"-"}"
  echo -e " --"
  echo -e " _CMD:\t\t\t\t${_CMD:-"-"}"
  echo -e " _ARGS:\t\t\t\t${_ARGS:-"-"}"
}

# Alias for info
_i() {
  _info
}

_gi() {
  _global_info
}

_li() {
  _set
  _local_info
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

    if ((_DRY_RUN)); then
      _log_info "Perform operation with --dry-run."
    fi

    if [ "${_LOCAL_ENV_LOADED}" == "1" ]; then
      _log_debug "Loaded .env.default and local .env > ${_DILLA_ROOT_DIR}/.env"
    else
      _log_notice "Loaded .env.default"
      _log_notice "To work with private repositories and packages you should create and fill .env file with GITLAB_TOKEN variables from ${_ENV_DEFAULT}."
    fi

    # Run command if exist.
    # shellcheck disable=SC2128
    __call="_${_CMD}"
    if [[ "$(type -t "${__call}")" == 'function' ]]; then
      $__call
    else
      # shellcheck disable=SC2128
      _log_error "Unknown command: ${_CMD}"
    fi
  fi
}

###############################################################################
# Main
###############################################################################

# Call `_main` after everything has been defined.
_init
_main "$@"
