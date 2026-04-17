#!/usr/bin/env sh

set -eu

router_log() {
  printf '[sdkwork-router] %s\n' "$*"
}

router_die() {
  printf '[sdkwork-router] ERROR: %s\n' "$*" >&2
  exit 1
}

router_active_bootstrap_profile() {
  if [ -n "${SDKWORK_BOOTSTRAP_PROFILE:-}" ]; then
    printf '%s' "$SDKWORK_BOOTSTRAP_PROFILE"
    return 0
  fi

  printf '%s' 'runtime configuration'
}

router_bootstrap_identity_hint_path() {
  if [ -z "${SDKWORK_BOOTSTRAP_DATA_DIR:-}" ] || [ -z "${SDKWORK_BOOTSTRAP_PROFILE:-}" ]; then
    printf '%s' ''
    return 0
  fi

  printf '%s/identities/%s.json' "$(router_portable_path "$SDKWORK_BOOTSTRAP_DATA_DIR")" "$SDKWORK_BOOTSTRAP_PROFILE"
}

router_script_dir() {
  CDPATH= cd -- "$(dirname -- "$1")" && pwd
}

router_repo_root() {
  SCRIPT_DIR="$1"
  CDPATH= cd -- "$SCRIPT_DIR/.." && pwd
}

router_resolve_absolute_path() {
  BASE_PATH="$1"
  CANDIDATE_PATH="$2"

  case "$CANDIDATE_PATH" in
    /*)
      TARGET_PATH="$CANDIDATE_PATH"
      ;;
    *)
      TARGET_PATH="$BASE_PATH/$CANDIDATE_PATH"
      ;;
  esac

  if [ -d "$TARGET_PATH" ]; then
    CDPATH= cd -- "$TARGET_PATH" && pwd
    return 0
  fi

  PARENT_DIR=$(dirname -- "$TARGET_PATH")
  LEAF_NAME=$(basename -- "$TARGET_PATH")

  if [ "$PARENT_DIR" = "$TARGET_PATH" ]; then
    printf '%s' "$TARGET_PATH"
    return 0
  fi

  if [ -d "$PARENT_DIR" ]; then
    RESOLVED_PARENT=$(CDPATH= cd -- "$PARENT_DIR" && pwd)
  else
    RESOLVED_PARENT=$(router_resolve_absolute_path "$BASE_PATH" "$PARENT_DIR")
  fi

  case "$RESOLVED_PARENT" in
    /)
      printf '/%s' "$LEAF_NAME"
      ;;
    *)
      printf '%s/%s' "$RESOLVED_PARENT" "$LEAF_NAME"
      ;;
  esac
}

router_default_install_home() {
  REPO_ROOT="$1"
  printf '%s/artifacts/install/sdkwork-api-router/current' "$REPO_ROOT"
}

router_normalize_install_mode() {
  MODE="$(printf '%s' "${1:-portable}" | tr '[:upper:]' '[:lower:]' | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
  case "$MODE" in
    system)
      printf '%s' 'system'
      ;;
    *)
      printf '%s' 'portable'
      ;;
  esac
}

router_release_manifest_path() {
  printf '%s/release-manifest.json' "$1"
}

router_release_manifest_string() {
  MANIFEST_FILE="$1"
  KEY="$2"
  if [ ! -f "$MANIFEST_FILE" ]; then
    return 1
  fi

  VALUE=$(sed -n "s/^[[:space:]]*\"$KEY\"[[:space:]]*:[[:space:]]*\"\\(.*\\)\"[[:space:]]*,\{0,1\}[[:space:]]*$/\\1/p" "$MANIFEST_FILE" | head -n 1)
  if [ -z "$VALUE" ]; then
    return 1
  fi

  VALUE=$(printf '%s' "$VALUE" | sed 's/\\"/"/g; s/\\\\/\\/g')
  printf '%s' "$VALUE"
}

router_default_system_config_root() {
  case "$(router_runtime_platform_name)" in
    darwin)
      printf '%s' '/Library/Application Support/sdkwork-api-router'
      ;;
    *)
      printf '%s' '/etc/sdkwork-api-router'
      ;;
  esac
}

router_default_system_data_root() {
  case "$(router_runtime_platform_name)" in
    darwin)
      printf '%s' '/Library/Application Support/sdkwork-api-router/data'
      ;;
    *)
      printf '%s' '/var/lib/sdkwork-api-router'
      ;;
  esac
}

router_default_system_log_root() {
  case "$(router_runtime_platform_name)" in
    darwin)
      printf '%s' '/Library/Logs/sdkwork-api-router'
      ;;
    *)
      printf '%s' '/var/log/sdkwork-api-router'
      ;;
  esac
}

router_default_system_run_root() {
  case "$(router_runtime_platform_name)" in
    darwin)
      printf '%s' '/Library/Application Support/sdkwork-api-router/run'
      ;;
    *)
      printf '%s' '/run/sdkwork-api-router'
      ;;
  esac
}

router_default_config_root() {
  RUNTIME_HOME="$1"
  INSTALL_MODE="$(router_normalize_install_mode "${2:-portable}")"
  if [ "$INSTALL_MODE" = 'system' ]; then
    router_default_system_config_root
    return 0
  fi

  printf '%s/config' "$RUNTIME_HOME"
}

router_default_data_root() {
  RUNTIME_HOME="$1"
  INSTALL_MODE="$(router_normalize_install_mode "${2:-portable}")"
  if [ "$INSTALL_MODE" = 'system' ]; then
    router_default_system_data_root
    return 0
  fi

  printf '%s/var/data' "$RUNTIME_HOME"
}

router_default_log_root() {
  RUNTIME_HOME="$1"
  INSTALL_MODE="$(router_normalize_install_mode "${2:-portable}")"
  if [ "$INSTALL_MODE" = 'system' ]; then
    router_default_system_log_root
    return 0
  fi

  printf '%s/var/log' "$RUNTIME_HOME"
}

router_default_run_root() {
  RUNTIME_HOME="$1"
  INSTALL_MODE="$(router_normalize_install_mode "${2:-portable}")"
  if [ "$INSTALL_MODE" = 'system' ]; then
    router_default_system_run_root
    return 0
  fi

  printf '%s/var/run' "$RUNTIME_HOME"
}

router_default_database_url() {
  DATA_ROOT="$1"
  INSTALL_MODE="$(router_normalize_install_mode "${2:-portable}")"
  if [ "$INSTALL_MODE" = 'system' ]; then
    printf '%s' 'postgresql://sdkwork:change-me@127.0.0.1:5432/sdkwork_api_router'
    return 0
  fi

  printf 'sqlite://%s/sdkwork-api-router.db' "$(router_portable_path "$DATA_ROOT")"
}

router_default_dev_home() {
  REPO_ROOT="$1"
  printf '%s/artifacts/runtime/dev/%s' "$REPO_ROOT" "$(router_runtime_key)"
}

router_is_windows() {
  case "$(uname -s 2>/dev/null || echo unknown)" in
    CYGWIN*|MINGW*|MSYS*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

router_is_wsl() {
  if router_is_windows; then
    return 1
  fi

  if [ -n "${WSL_DISTRO_NAME:-}" ] || [ -n "${WSL_INTEROP:-}" ]; then
    return 0
  fi

  if [ -r /proc/sys/kernel/osrelease ] && grep -qi 'microsoft' /proc/sys/kernel/osrelease 2>/dev/null; then
    return 0
  fi

  if [ -r /proc/version ] && grep -qi 'microsoft' /proc/version 2>/dev/null; then
    return 0
  fi

  return 1
}

router_is_interactive_shell() {
  case "$-" in
    *i*)
      return 0
      ;;
  esac

  if [ -t 0 ] || [ -t 1 ]; then
    return 0
  fi

  return 1
}

router_is_wsl_windows_mount_path() {
  case "$1" in
    /mnt/[A-Za-z]|/mnt/[A-Za-z]/*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

router_warn_wsl_background_session() {
  LABEL="$1"

  if ! router_is_wsl || router_is_interactive_shell; then
    return 0
  fi

  router_log "WARNING: $LABEL is starting from a non-interactive WSL session."
  router_log 'WARNING: Background services launched from one-shot wsl.exe commands may stop when the session exits.'
  router_log 'WARNING: Run this script from an interactive WSL shell or use --foreground for long-lived sessions.'
}

router_runtime_platform_name() {
  case "$(uname -s 2>/dev/null || echo unknown)" in
    CYGWIN*|MINGW*|MSYS*)
      printf '%s' 'windows'
      ;;
    Darwin*)
      printf '%s' 'macos'
      ;;
    Linux*)
      printf '%s' 'linux'
      ;;
    *)
      printf '%s' 'unknown'
      ;;
  esac
}

router_runtime_arch_name() {
  case "$(uname -m 2>/dev/null || echo unknown)" in
    x86_64|amd64)
      printf '%s' 'x64'
      ;;
    aarch64|arm64)
      printf '%s' 'arm64'
      ;;
    i386|i486|i586|i686|x86)
      printf '%s' 'x86'
      ;;
    armv7l|armv7|armhf)
      printf '%s' 'armv7'
      ;;
    *)
      printf '%s' "$(uname -m 2>/dev/null || echo unknown)" | tr '[:upper:]' '[:lower:]'
      ;;
  esac
}

router_runtime_key() {
  printf '%s-%s' "$(router_runtime_platform_name)" "$(router_runtime_arch_name)"
}

router_binary_name() {
  NAME="$1"
  if router_is_windows; then
    printf '%s.exe' "$NAME"
    return 0
  fi

  printf '%s' "$NAME"
}

router_portable_path() {
  printf '%s' "$1" | sed 's#\\#/#g'
}

router_windows_path() {
  if command -v cygpath >/dev/null 2>&1; then
    cygpath -w "$1"
    return 0
  fi

  printf '%s' "$1"
}

router_windows_cli_path() {
  VALUE="$1"
  if [ -z "$VALUE" ]; then
    printf '%s' ''
    return 0
  fi

  if router_is_windows_path "$VALUE"; then
    printf '%s' "$(router_portable_path "$VALUE")"
    return 0
  fi

  if router_is_unix_absolute_path "$VALUE"; then
    printf '%s' "$(router_portable_path "$(router_windows_path "$VALUE")")"
    return 0
  fi

  printf '%s' "$VALUE"
}

router_windows_database_url() {
  DATABASE_URL="$1"
  case "$DATABASE_URL" in
    sqlite://*)
      DATABASE_PATH=${DATABASE_URL#sqlite://}
      if router_is_windows_path "$DATABASE_PATH"; then
        printf 'sqlite://%s' "$(router_portable_path "$DATABASE_PATH")"
        return 0
      fi
      if router_is_unix_absolute_path "$DATABASE_PATH"; then
        printf 'sqlite://%s' "$(router_windows_cli_path "$DATABASE_PATH")"
        return 0
      fi
      ;;
  esac

  printf '%s' "$DATABASE_URL"
}

router_powershell_quote() {
  printf '%s' "$1" | sed "s/'/''/g"
}

router_ensure_dir() {
  mkdir -p "$1"
}

router_trim() {
  printf '%s' "$1" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//'
}

router_is_windows_path() {
  case "$1" in
    [A-Za-z]:/*|[A-Za-z]:\\*|\\\\*|//*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

router_is_unix_absolute_path() {
  case "$1" in
    /*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

router_path_is_host_compatible() {
  VALUE="$1"
  if [ -z "$VALUE" ]; then
    return 0
  fi

  if router_is_windows; then
    if router_is_unix_absolute_path "$VALUE" && ! router_is_windows_path "$VALUE"; then
      return 1
    fi
    return 0
  fi

  if router_is_windows_path "$VALUE"; then
    return 1
  fi

  return 0
}

router_resolve_host_path() {
  VALUE="$1"
  DEFAULT_VALUE="$2"
  if router_path_is_host_compatible "$VALUE"; then
    printf '%s' "$VALUE"
    return 0
  fi

  printf '%s' "$DEFAULT_VALUE"
}

router_database_url_is_host_compatible() {
  DATABASE_URL="$1"
  case "$DATABASE_URL" in
    sqlite://*)
      DATABASE_PATH=${DATABASE_URL#sqlite://}
      router_path_is_host_compatible "$DATABASE_PATH"
      return $?
      ;;
    *)
      return 0
      ;;
  esac
}

router_resolve_host_database_url() {
  DATABASE_URL="$1"
  DEFAULT_VALUE="$2"
  if router_database_url_is_host_compatible "$DATABASE_URL"; then
    printf '%s' "$DATABASE_URL"
    return 0
  fi

  printf '%s' "$DEFAULT_VALUE"
}

router_bind_port() {
  BIND_ADDRESS="$1"
  printf '%s' "${BIND_ADDRESS##*:}"
}

router_collect_bind_conflicts_windows() {
  if ! command -v powershell.exe >/dev/null 2>&1; then
    router_log 'WARNING: unable to preflight port conflicts because powershell.exe is unavailable.'
    return 0
  fi

  if [ -z "${SCRIPT_DIR:-}" ] || [ ! -f "$SCRIPT_DIR/lib/runtime-common.ps1" ]; then
    router_log 'WARNING: unable to preflight port conflicts because runtime-common.ps1 is unavailable.'
    return 0
  fi

  ARGS_FILE=$(mktemp "${TMPDIR:-/tmp}/sdkwork-router-bind-args.XXXXXX")
  : > "$ARGS_FILE"
  for BIND_ADDRESS in "$@"; do
    printf '%s\n' "$BIND_ADDRESS" >> "$ARGS_FILE"
  done

  POWERSHELL_HELPER_WIN=$(router_powershell_quote "$(router_windows_path "$SCRIPT_DIR/lib/runtime-common.ps1")")
  ARGS_FILE_WIN=$(router_powershell_quote "$(router_windows_path "$ARGS_FILE")")

  powershell.exe -NoProfile -ExecutionPolicy Bypass -Command ". '$POWERSHELL_HELPER_WIN'; \$bindAddresses = @(); if (Test-Path '$ARGS_FILE_WIN') { \$bindAddresses = @(Get-Content '$ARGS_FILE_WIN' -ErrorAction SilentlyContinue) }; \$conflicts = @(Get-RouterListeningPortConflicts -BindAddresses \$bindAddresses); foreach (\$conflict in \$conflicts) { Write-Output (\$conflict.BindAddress + '|' + \$conflict.Reason) }" 2>/dev/null || true

  rm -f "$ARGS_FILE"
}

router_collect_bind_conflicts_unix() {
  for BIND_ADDRESS in "$@"; do
    PORT="$(router_bind_port "$BIND_ADDRESS")"
    DETAILS=''

    if command -v lsof >/dev/null 2>&1; then
      DETAILS=$(lsof -nP -iTCP:"$PORT" -sTCP:LISTEN 2>/dev/null | awk 'NR > 1 { printf "%s(pid=%s)%s", $1, $2, ORS }')
    elif command -v ss >/dev/null 2>&1; then
      DETAILS=$(ss -ltnp "( sport = :$PORT )" 2>/dev/null | awk 'NR > 1 { print }')
    elif command -v netstat >/dev/null 2>&1; then
      DETAILS=$(netstat -an 2>/dev/null | grep "[\.\:]$PORT" | grep LISTEN || true)
    fi

    if [ -n "$DETAILS" ]; then
      printf '%s|%s\n' "$BIND_ADDRESS" "$DETAILS"
    fi
  done
}

router_collect_bind_conflicts() {
  if [ "$#" -eq 0 ]; then
    return 0
  fi

  if router_is_windows; then
    router_collect_bind_conflicts_windows "$@"
    return 0
  fi

  if command -v lsof >/dev/null 2>&1 || command -v ss >/dev/null 2>&1 || command -v netstat >/dev/null 2>&1; then
    router_collect_bind_conflicts_unix "$@"
    return 0
  fi

  router_log 'WARNING: unable to preflight port conflicts because lsof, ss, and netstat are unavailable.'
}

router_assert_bind_addresses_available() {
  SERVICE_LABEL="$1"
  shift

  CONFLICT_LINES=$(router_collect_bind_conflicts "$@" || true)
  if [ -z "$CONFLICT_LINES" ]; then
    return 0
  fi

  MESSAGE="$SERVICE_LABEL cannot start because required listen ports are already in use:"
  OLD_IFS=$IFS
  IFS='
'
  for CONFLICT_LINE in $CONFLICT_LINES; do
    [ -n "$CONFLICT_LINE" ] || continue
    BIND_ADDRESS=${CONFLICT_LINE%%|*}
    DETAILS=${CONFLICT_LINE#*|}
    MESSAGE="$MESSAGE
  $BIND_ADDRESS ($DETAILS)"
  done
  IFS=$OLD_IFS

  MESSAGE="$MESSAGE
Stop the conflicting process or override the bind addresses before retrying."
  router_die "$MESSAGE"
}

router_json_escape() {
  printf '%s' "$1" | sed ':a;N;$!ba;s/\\/\\\\/g;s/"/\\"/g;s/\r/\\r/g;s/\n/\\n/g;s/\t/\\t/g'
}

router_json_string() {
  printf '"%s"' "$(router_json_escape "$1")"
}

router_json_nullable_string() {
  if [ -n "$1" ]; then
    router_json_string "$1"
    return 0
  fi

  printf '%s' 'null'
}

router_json_roles_value() {
  ROLES_VALUE="${1:-}"
  if [ -z "$ROLES_VALUE" ]; then
    printf '%s' '["web","gateway","admin","portal"]'
    return 0
  fi

  OUTPUT='['
  FIRST='1'
  OLD_IFS=$IFS
  IFS=',;'
  set -- $ROLES_VALUE
  IFS=$OLD_IFS

  for ROLE in "$@"; do
    ROLE=$(router_trim "$ROLE")
    if [ -z "$ROLE" ]; then
      continue
    fi
    if [ "$FIRST" != '1' ]; then
      OUTPUT="$OUTPUT,"
    fi
    FIRST='0'
    OUTPUT="$OUTPUT\"$(router_json_escape "$ROLE")\""
  done

  if [ "$FIRST" = '1' ]; then
    printf '%s' '["web","gateway","admin","portal"]'
    return 0
  fi

  printf '%s]' "$OUTPUT"
}

router_render_release_dry_run_plan_json() {
  CONFIG_DIR="$1"
  DATABASE_URL="$2"
  WEB_BIND="$3"
  GATEWAY_BIND="$4"
  ADMIN_BIND="$5"
  PORTAL_BIND="$6"
  CONFIG_FILE="${7:-}"
  ROLES_VALUE="${8:-}"
  NODE_ID_PREFIX="${9:-}"
  GATEWAY_UPSTREAM="${10:-}"
  ADMIN_UPSTREAM="${11:-}"
  PORTAL_UPSTREAM="${12:-}"
  ADMIN_SITE_DIR="${13:-}"
  PORTAL_SITE_DIR="${14:-}"

  cat <<EOF
{
  "mode": "dry-run",
  "plan_format": "json",
  "roles": $(router_json_roles_value "$ROLES_VALUE"),
  "public_web_bind": $(router_json_string "$WEB_BIND"),
  "database_url": $(router_json_string "$DATABASE_URL"),
  "config_dir": $(router_json_string "$CONFIG_DIR"),
  "config_file": $(router_json_nullable_string "$CONFIG_FILE"),
  "node_id_prefix": $(router_json_nullable_string "$NODE_ID_PREFIX"),
  "binds": {
    "gateway": $(router_json_string "$GATEWAY_BIND"),
    "admin": $(router_json_string "$ADMIN_BIND"),
    "portal": $(router_json_string "$PORTAL_BIND")
  },
  "site_dirs": {
    "admin": $(router_json_string "$ADMIN_SITE_DIR"),
    "portal": $(router_json_string "$PORTAL_SITE_DIR")
  },
  "upstreams": {
    "gateway": $(router_json_nullable_string "$GATEWAY_UPSTREAM"),
    "admin": $(router_json_nullable_string "$ADMIN_UPSTREAM"),
    "portal": $(router_json_nullable_string "$PORTAL_UPSTREAM")
  }
}
EOF
}

router_unquote_env_value() {
  VALUE="$1"

  case "$VALUE" in
    \"*\")
      VALUE=${VALUE#\"}
      VALUE=${VALUE%\"}
      VALUE=$(printf '%s' "$VALUE" | sed 's/\\"/"/g; s/\\\\/\\/g')
      ;;
    \'*\')
      VALUE=${VALUE#\'}
      VALUE=${VALUE%\'}
      ;;
  esac

  printf '%s' "$VALUE"
}

router_load_env_file() {
  ENV_FILE="$1"
  if [ ! -f "$ENV_FILE" ]; then
    return 0
  fi

  while IFS= read -r RAW_LINE || [ -n "$RAW_LINE" ]; do
    LINE="$(router_trim "$RAW_LINE")"
    case "$LINE" in
      ''|'#'*)
        continue
        ;;
    esac

    KEY="${LINE%%=*}"
    VALUE="${LINE#*=}"
    KEY="$(router_trim "$KEY")"
    VALUE="$(router_trim "$VALUE")"
    VALUE="$(router_unquote_env_value "$VALUE")"
    export "$KEY=$VALUE"
  done < "$ENV_FILE"
}

router_clear_managed_state_env() {
  unset SDKWORK_ROUTER_MANAGED_PID || true
  unset SDKWORK_ROUTER_PROCESS_FINGERPRINT || true
  unset SDKWORK_ROUTER_MANAGED_MODE || true
  unset SDKWORK_ROUTER_MANAGED_WEB_BIND || true
  unset SDKWORK_ROUTER_MANAGED_GATEWAY_BIND || true
  unset SDKWORK_ROUTER_MANAGED_ADMIN_BIND || true
  unset SDKWORK_ROUTER_MANAGED_PORTAL_BIND || true
  unset SDKWORK_ROUTER_MANAGED_UNIFIED_ACCESS_ENABLED || true
  unset SDKWORK_ROUTER_MANAGED_ADMIN_APP_URL || true
  unset SDKWORK_ROUTER_MANAGED_PORTAL_APP_URL || true
}

router_get_process_fingerprint() {
  PID="$1"
  if [ -z "$PID" ] || ! router_is_pid_running "$PID"; then
    printf '%s' ''
    return 1
  fi

  if router_is_windows; then
    powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "(Get-Process -Id $PID -ErrorAction SilentlyContinue).StartTime.ToUniversalTime().ToString('o')" 2>/dev/null | tr -d '\r'
    return 0
  fi

  if command -v ps >/dev/null 2>&1; then
    ps -o lstart= -p "$PID" 2>/dev/null | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' | head -n 1
    return 0
  fi

  printf '%s' ''
}

router_state_escape() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

router_write_state_line() {
  KEY="$1"
  VALUE="$2"
  printf '%s="%s"\n' "$KEY" "$(router_state_escape "$VALUE")"
}

router_read_managed_state() {
  STATE_FILE="$1"

  router_clear_managed_state_env
  if [ ! -f "$STATE_FILE" ]; then
    return 1
  fi

  router_load_env_file "$STATE_FILE"
  [ -n "${SDKWORK_ROUTER_MANAGED_PID:-}" ]
}

router_write_managed_state() {
  STATE_FILE="$1"
  PROCESS_ID="$2"
  PROCESS_FINGERPRINT="$3"
  MODE="$4"
  WEB_BIND="$5"
  GATEWAY_BIND="$6"
  ADMIN_BIND="$7"
  PORTAL_BIND="$8"
  UNIFIED_ACCESS_ENABLED="$9"
  ADMIN_APP_URL="${10:-}"
  PORTAL_APP_URL="${11:-}"

  STATE_DIR=$(dirname "$STATE_FILE")
  [ "$STATE_DIR" = '.' ] || router_ensure_dir "$STATE_DIR"

  {
    router_write_state_line 'SDKWORK_ROUTER_MANAGED_PID' "$PROCESS_ID"
    router_write_state_line 'SDKWORK_ROUTER_PROCESS_FINGERPRINT' "$PROCESS_FINGERPRINT"
    router_write_state_line 'SDKWORK_ROUTER_MANAGED_MODE' "$MODE"
    router_write_state_line 'SDKWORK_ROUTER_MANAGED_WEB_BIND' "$WEB_BIND"
    router_write_state_line 'SDKWORK_ROUTER_MANAGED_GATEWAY_BIND' "$GATEWAY_BIND"
    router_write_state_line 'SDKWORK_ROUTER_MANAGED_ADMIN_BIND' "$ADMIN_BIND"
    router_write_state_line 'SDKWORK_ROUTER_MANAGED_PORTAL_BIND' "$PORTAL_BIND"
    router_write_state_line 'SDKWORK_ROUTER_MANAGED_UNIFIED_ACCESS_ENABLED' "$UNIFIED_ACCESS_ENABLED"
    router_write_state_line 'SDKWORK_ROUTER_MANAGED_ADMIN_APP_URL' "$ADMIN_APP_URL"
    router_write_state_line 'SDKWORK_ROUTER_MANAGED_PORTAL_APP_URL' "$PORTAL_APP_URL"
  } > "$STATE_FILE"
}

router_remove_managed_state() {
  STATE_FILE="${1:-}"
  [ -n "$STATE_FILE" ] && rm -f "$STATE_FILE"
  router_clear_managed_state_env
}

router_is_pid_running() {
  PID="$1"
  if [ -z "$PID" ]; then
    return 1
  fi

  if router_is_windows; then
    powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "if (Get-Process -Id $PID -ErrorAction SilentlyContinue) { exit 0 } else { exit 1 }" >/dev/null 2>&1
    return $?
  fi

  kill -0 "$PID" 2>/dev/null
}

router_cleanup_stale_pid_file() {
  PID_FILE="$1"
  STATE_FILE="${2:-}"
  if [ ! -f "$PID_FILE" ]; then
    router_remove_managed_state "$STATE_FILE"
    return 0
  fi

  PID="$(tr -d '[:space:]' < "$PID_FILE" 2>/dev/null || true)"
  if [ -z "$PID" ]; then
    rm -f "$PID_FILE"
    router_remove_managed_state "$STATE_FILE"
    return 0
  fi

  if ! router_is_pid_running "$PID"; then
    rm -f "$PID_FILE"
    router_remove_managed_state "$STATE_FILE"
    return 0
  fi

  if [ -n "$STATE_FILE" ] && [ -f "$STATE_FILE" ]; then
    if ! router_read_managed_state "$STATE_FILE"; then
      rm -f "$PID_FILE"
      router_remove_managed_state "$STATE_FILE"
      return 0
    fi

    if [ "${SDKWORK_ROUTER_MANAGED_PID:-}" != "$PID" ]; then
      rm -f "$PID_FILE"
      router_remove_managed_state "$STATE_FILE"
      return 0
    fi

    CURRENT_FINGERPRINT=$(router_get_process_fingerprint "$PID")
    if [ -n "${SDKWORK_ROUTER_PROCESS_FINGERPRINT:-}" ] && [ "${SDKWORK_ROUTER_PROCESS_FINGERPRINT}" != "$CURRENT_FINGERPRINT" ]; then
      rm -f "$PID_FILE"
      router_remove_managed_state "$STATE_FILE"
      return 0
    fi
  fi

  return 1
}

router_get_running_pid() {
  PID_FILE="$1"
  STATE_FILE="${2:-}"
  if [ ! -f "$PID_FILE" ]; then
    router_remove_managed_state "$STATE_FILE"
    printf '%s' ''
    return 0
  fi

  PID="$(tr -d '[:space:]' < "$PID_FILE" 2>/dev/null || true)"
  if [ -z "$PID" ]; then
    rm -f "$PID_FILE"
    router_remove_managed_state "$STATE_FILE"
    printf '%s' ''
    return 0
  fi

  if ! router_is_pid_running "$PID"; then
    rm -f "$PID_FILE"
    router_remove_managed_state "$STATE_FILE"
    printf '%s' ''
    return 0
  fi

  if [ -n "$STATE_FILE" ] && [ -f "$STATE_FILE" ]; then
    if ! router_read_managed_state "$STATE_FILE"; then
      rm -f "$PID_FILE"
      router_remove_managed_state "$STATE_FILE"
      printf '%s' ''
      return 0
    fi

    if [ "${SDKWORK_ROUTER_MANAGED_PID:-}" != "$PID" ]; then
      rm -f "$PID_FILE"
      router_remove_managed_state "$STATE_FILE"
      printf '%s' ''
      return 0
    fi

    CURRENT_FINGERPRINT=$(router_get_process_fingerprint "$PID")
    if [ -n "${SDKWORK_ROUTER_PROCESS_FINGERPRINT:-}" ] && [ "${SDKWORK_ROUTER_PROCESS_FINGERPRINT}" != "$CURRENT_FINGERPRINT" ]; then
      rm -f "$PID_FILE"
      router_remove_managed_state "$STATE_FILE"
      printf '%s' ''
      return 0
    fi
  fi

  printf '%s' "$PID"
}

router_require_not_running() {
  PID_FILE="$1"
  STATE_FILE="${2:-}"
  PID="$(router_get_running_pid "$PID_FILE" "$STATE_FILE")"
  if [ -z "$PID" ]; then
    return 0
  fi

  router_die "process already running with pid $PID (pid file: $PID_FILE)"
}

router_wait_for_pid_exit() {
  PID="$1"
  WAIT_SECONDS="$2"
  COUNTER=0
  while router_is_pid_running "$PID"; do
    if [ "$COUNTER" -ge "$WAIT_SECONDS" ]; then
      return 1
    fi
    sleep 1
    COUNTER=$((COUNTER + 1))
  done
  return 0
}

router_confirm_pid_alive() {
  PID="$1"
  WAIT_SECONDS="${2:-2}"
  COUNTER=0

  while [ "$COUNTER" -lt "$WAIT_SECONDS" ]; do
    if ! router_is_pid_running "$PID"; then
      return 1
    fi
    sleep 1
    COUNTER=$((COUNTER + 1))
  done

  return 0
}

router_start_background_process() {
  FILE_PATH="$1"
  WORKING_DIR="$2"
  STDOUT_LOG="$3"
  STDERR_LOG="$4"
  shift 4

  if router_is_windows; then
    FILE_PATH_WIN=$(router_powershell_quote "$(router_windows_path "$FILE_PATH")")
    WORKING_DIR_WIN=$(router_powershell_quote "$(router_windows_path "$WORKING_DIR")")
    STDOUT_LOG_WIN=$(router_powershell_quote "$(router_windows_path "$STDOUT_LOG")")
    STDERR_LOG_WIN=$(router_powershell_quote "$(router_windows_path "$STDERR_LOG")")
    ARGS_FILE=$(mktemp "${TMPDIR:-/tmp}/sdkwork-router-args.XXXXXX")
    PID_CAPTURE_FILE=$(mktemp "${TMPDIR:-/tmp}/sdkwork-router-pid.XXXXXX")
    : > "$ARGS_FILE"
    for ARG in "$@"; do
      printf '%s\n' "$ARG" >> "$ARGS_FILE"
    done
    ARGS_FILE_WIN=$(router_powershell_quote "$(router_windows_path "$ARGS_FILE")")
    PID_CAPTURE_FILE_WIN=$(router_powershell_quote "$(router_windows_path "$PID_CAPTURE_FILE")")
    powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "\$argsFile = '$ARGS_FILE_WIN'; \$pidFile = '$PID_CAPTURE_FILE_WIN'; \$argumentList = @(); if (Test-Path \$argsFile) { \$argumentList = @(Get-Content \$argsFile -ErrorAction SilentlyContinue) }; \$startArgs = @{ FilePath = '$FILE_PATH_WIN'; WorkingDirectory = '$WORKING_DIR_WIN'; RedirectStandardOutput = '$STDOUT_LOG_WIN'; RedirectStandardError = '$STDERR_LOG_WIN'; WindowStyle = 'Hidden'; PassThru = \$true }; if (\$argumentList.Count -gt 0) { \$startArgs.ArgumentList = \$argumentList }; \$process = Start-Process @startArgs; [System.IO.File]::WriteAllText(\$pidFile, [string]\$process.Id)" >/dev/null 2>&1
    STARTED_PID=$(tr -d '\r\n[:space:]' < "$PID_CAPTURE_FILE" 2>/dev/null || true)
    rm -f "$ARGS_FILE"
    rm -f "$PID_CAPTURE_FILE"
    printf '%s' "$STARTED_PID"
    return 0
  fi

  nohup "$FILE_PATH" "$@" >> "$STDOUT_LOG" 2>> "$STDERR_LOG" &
  printf '%s' "$!"
}

router_stop_pid() {
  PID="$1"
  WAIT_SECONDS="$2"
  FORCE_MODE="$3"

  if ! router_is_pid_running "$PID"; then
    return 0
  fi

  if router_is_windows; then
    cmd.exe /c "taskkill /PID $PID /T >nul 2>nul" >/dev/null 2>&1 || true
    if router_wait_for_pid_exit "$PID" "$WAIT_SECONDS"; then
      return 0
    fi

    if [ "$FORCE_MODE" != '1' ]; then
      return 1
    fi

    cmd.exe /c "taskkill /PID $PID /T /F >nul 2>nul" >/dev/null 2>&1 || true
    router_wait_for_pid_exit "$PID" "$WAIT_SECONDS" || return 1
    return 0
  fi

  kill "$PID" 2>/dev/null || true
  if router_wait_for_pid_exit "$PID" "$WAIT_SECONDS"; then
    return 0
  fi

  if [ "$FORCE_MODE" != '1' ]; then
    return 1
  fi

  kill -9 "$PID" 2>/dev/null || true
  router_wait_for_pid_exit "$PID" "$WAIT_SECONDS" || return 1
}

router_resolve_loopback_url() {
  BIND_ADDR="$1"
  REQUEST_PATH="$2"
  HOST="${BIND_ADDR%:*}"
  PORT="${BIND_ADDR##*:}"

  case "$HOST" in
    ''|'0.0.0.0'|'[::]'|'::')
      HOST='127.0.0.1'
      ;;
  esac

  printf 'http://%s:%s%s' "$HOST" "$PORT" "$REQUEST_PATH"
}

router_http_ready() {
  URL="$1"
  if command -v curl >/dev/null 2>&1; then
    curl --silent --show-error --fail --max-time 3 "$URL" >/dev/null 2>&1
    return $?
  fi

  router_die "curl is required for health checks in shell scripts"
}

router_wait_for_url() {
  URL="$1"
  WAIT_SECONDS="$2"
  WATCH_PID="${3:-}"
  COUNTER=0
  while ! router_http_ready "$URL"; do
    if [ -n "$WATCH_PID" ] && ! router_is_pid_running "$WATCH_PID"; then
      return 1
    fi
    if [ "$COUNTER" -ge "$WAIT_SECONDS" ]; then
      return 1
    fi
    sleep 1
    COUNTER=$((COUNTER + 1))
  done
  if [ -n "$WATCH_PID" ] && ! router_is_pid_running "$WATCH_PID"; then
    return 1
  fi
  return 0
}

router_tail_log() {
  LOG_FILE="$1"
  if [ -f "$LOG_FILE" ]; then
    tail -n 60 "$LOG_FILE" 2>/dev/null || true
  fi
}

router_validate_file() {
  LABEL="$1"
  FILE_PATH="$2"
  if [ ! -f "$FILE_PATH" ]; then
    router_die "$LABEL not found: $FILE_PATH"
  fi
}

router_validate_dir() {
  LABEL="$1"
  DIR_PATH="$2"
  if [ ! -d "$DIR_PATH" ]; then
    router_die "$LABEL not found: $DIR_PATH"
  fi
}

router_log_detail() {
  LABEL="$1"
  VALUE="$2"
  router_log "  $LABEL: $VALUE"
}

router_startup_summary() {
  MODE="$1"
  UNIFIED_ACCESS_ENABLED="$2"
  WEB_BIND="$3"
  GATEWAY_BIND="$4"
  ADMIN_BIND="$5"
  PORTAL_BIND="$6"
  ADMIN_APP_URL="$7"
  PORTAL_APP_URL="$8"
  STDOUT_LOG="$9"
  STDERR_LOG="${10}"

  [ -n "$ADMIN_APP_URL" ] || ADMIN_APP_URL=$(router_resolve_loopback_url "$WEB_BIND" "/admin/")
  [ -n "$PORTAL_APP_URL" ] || PORTAL_APP_URL=$(router_resolve_loopback_url "$WEB_BIND" "/portal/")

  GATEWAY_UNIFIED_URL=$(router_resolve_loopback_url "$WEB_BIND" "/api/v1/health")
  ADMIN_UNIFIED_URL=$(router_resolve_loopback_url "$WEB_BIND" "/api/admin/health")
  PORTAL_UNIFIED_URL=$(router_resolve_loopback_url "$WEB_BIND" "/api/portal/health")
  GATEWAY_DIRECT_URL=$(router_resolve_loopback_url "$GATEWAY_BIND" "/health")
  ADMIN_DIRECT_URL=$(router_resolve_loopback_url "$ADMIN_BIND" "/admin/health")
  PORTAL_DIRECT_URL=$(router_resolve_loopback_url "$PORTAL_BIND" "/portal/health")
  GATEWAY_OPENAPI_URL=$(router_resolve_loopback_url "$GATEWAY_BIND" "/openapi.json")
  ADMIN_OPENAPI_URL=$(router_resolve_loopback_url "$ADMIN_BIND" "/admin/openapi.json")
  PORTAL_OPENAPI_URL=$(router_resolve_loopback_url "$PORTAL_BIND" "/portal/openapi.json")
  BOOTSTRAP_PROFILE=$(router_active_bootstrap_profile)
  BOOTSTRAP_IDENTITY_HINT_PATH=$(router_bootstrap_identity_hint_path)

  router_log '------------------------------------------------------------'
  router_log "Mode: $MODE"
  router_log "Bind Summary: web=$WEB_BIND gateway=$GATEWAY_BIND admin=$ADMIN_BIND portal=$PORTAL_BIND"

  if [ "$UNIFIED_ACCESS_ENABLED" = '1' ]; then
    router_log 'Unified Access'
    router_log_detail 'Admin App' "$ADMIN_APP_URL"
    router_log_detail 'Portal App' "$PORTAL_APP_URL"
    router_log_detail 'Gateway API Health' "$GATEWAY_UNIFIED_URL"
    router_log_detail 'Admin API Health' "$ADMIN_UNIFIED_URL"
    router_log_detail 'Portal API Health' "$PORTAL_UNIFIED_URL"
  else
    router_log 'Frontend Access'
    router_log_detail 'Admin App' "$ADMIN_APP_URL"
    router_log_detail 'Portal App' "$PORTAL_APP_URL"
  fi

  router_log 'Direct Service Access'
  router_log_detail 'Gateway Service' "$GATEWAY_DIRECT_URL"
  router_log_detail 'Admin Service' "$ADMIN_DIRECT_URL"
  router_log_detail 'Portal Service' "$PORTAL_DIRECT_URL"
  router_log_detail 'Gateway OpenAPI 3.x Schema' "$GATEWAY_OPENAPI_URL"
  router_log_detail 'Admin OpenAPI 3.x Schema' "$ADMIN_OPENAPI_URL"
  router_log_detail 'Portal OpenAPI 3.x Schema' "$PORTAL_OPENAPI_URL"

  router_log 'Identity Bootstrap'
  router_log_detail 'Local access' "uses the active bootstrap profile: $BOOTSTRAP_PROFILE"
  if [ -n "$BOOTSTRAP_IDENTITY_HINT_PATH" ]; then
    router_log_detail 'Identity source' "review your runtime configuration and provisioned identities in $BOOTSTRAP_IDENTITY_HINT_PATH before sharing the environment."
  else
    router_log_detail 'Identity source' 'review your runtime configuration and provisioned identity store before sharing the environment.'
  fi
  router_log_detail 'Portal sign-in' 'use a provisioned portal user or register through /portal/auth/register.'
  router_log_detail 'Gateway API' 'sign in through the portal and create an API key.'

  router_log 'Logs'
  router_log_detail 'STDOUT' "$STDOUT_LOG"
  router_log_detail 'STDERR' "$STDERR_LOG"
}
