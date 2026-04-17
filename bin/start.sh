#!/usr/bin/env sh

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
# shellcheck source=bin/lib/runtime-common.sh
. "$SCRIPT_DIR/lib/runtime-common.sh"

REPO_ROOT=$(router_repo_root "$SCRIPT_DIR")
DEFAULT_HOME=$(router_default_install_home "$REPO_ROOT")

RUNTIME_HOME=''
FOREGROUND=0
DRY_RUN=0
WAIT_SECONDS=60

CLI_BIND=''
CLI_CONFIG_DIR=''
CLI_CONFIG_FILE=''
CLI_DATABASE_URL=''
CLI_ROLES=''
CLI_NODE_ID_PREFIX=''
CLI_GATEWAY_BIND=''
CLI_ADMIN_BIND=''
CLI_PORTAL_BIND=''
CLI_GATEWAY_UPSTREAM=''
CLI_ADMIN_UPSTREAM=''
CLI_PORTAL_UPSTREAM=''
CLI_ADMIN_SITE_DIR=''
CLI_PORTAL_SITE_DIR=''

while [ "$#" -gt 0 ]; do
  case "$1" in
    --home)
      [ "$#" -ge 2 ] || router_die "--home requires a value"
      RUNTIME_HOME="$2"
      shift 2
      ;;
    --foreground)
      FOREGROUND=1
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --wait-seconds)
      [ "$#" -ge 2 ] || router_die "--wait-seconds requires a value"
      WAIT_SECONDS="$2"
      shift 2
      ;;
    --bind)
      [ "$#" -ge 2 ] || router_die "--bind requires a value"
      CLI_BIND="$2"
      shift 2
      ;;
    --config-dir)
      [ "$#" -ge 2 ] || router_die "--config-dir requires a value"
      CLI_CONFIG_DIR="$2"
      shift 2
      ;;
    --config-file)
      [ "$#" -ge 2 ] || router_die "--config-file requires a value"
      CLI_CONFIG_FILE="$2"
      shift 2
      ;;
    --database-url)
      [ "$#" -ge 2 ] || router_die "--database-url requires a value"
      CLI_DATABASE_URL="$2"
      shift 2
      ;;
    --roles)
      [ "$#" -ge 2 ] || router_die "--roles requires a value"
      CLI_ROLES="$2"
      shift 2
      ;;
    --node-id-prefix)
      [ "$#" -ge 2 ] || router_die "--node-id-prefix requires a value"
      CLI_NODE_ID_PREFIX="$2"
      shift 2
      ;;
    --gateway-bind)
      [ "$#" -ge 2 ] || router_die "--gateway-bind requires a value"
      CLI_GATEWAY_BIND="$2"
      shift 2
      ;;
    --admin-bind)
      [ "$#" -ge 2 ] || router_die "--admin-bind requires a value"
      CLI_ADMIN_BIND="$2"
      shift 2
      ;;
    --portal-bind)
      [ "$#" -ge 2 ] || router_die "--portal-bind requires a value"
      CLI_PORTAL_BIND="$2"
      shift 2
      ;;
    --gateway-upstream)
      [ "$#" -ge 2 ] || router_die "--gateway-upstream requires a value"
      CLI_GATEWAY_UPSTREAM="$2"
      shift 2
      ;;
    --admin-upstream)
      [ "$#" -ge 2 ] || router_die "--admin-upstream requires a value"
      CLI_ADMIN_UPSTREAM="$2"
      shift 2
      ;;
    --portal-upstream)
      [ "$#" -ge 2 ] || router_die "--portal-upstream requires a value"
      CLI_PORTAL_UPSTREAM="$2"
      shift 2
      ;;
    --admin-site-dir)
      [ "$#" -ge 2 ] || router_die "--admin-site-dir requires a value"
      CLI_ADMIN_SITE_DIR="$2"
      shift 2
      ;;
    --portal-site-dir)
      [ "$#" -ge 2 ] || router_die "--portal-site-dir requires a value"
      CLI_PORTAL_SITE_DIR="$2"
      shift 2
      ;;
    *)
      router_die "unknown option: $1"
      ;;
    esac
done

if router_is_windows; then
  PS_SCRIPT="$(router_windows_path "$SCRIPT_DIR/start.ps1")"
  set --
  [ -n "$RUNTIME_HOME" ] && set -- "$@" -Home "$(router_windows_cli_path "$RUNTIME_HOME")"
  [ "$FOREGROUND" = '1' ] && set -- "$@" -Foreground
  [ "$DRY_RUN" = '1' ] && set -- "$@" -DryRun
  [ "$WAIT_SECONDS" != '60' ] && set -- "$@" -WaitSeconds "$WAIT_SECONDS"
  [ -n "$CLI_BIND" ] && set -- "$@" -Bind "$CLI_BIND"
  [ -n "$CLI_CONFIG_DIR" ] && set -- "$@" -ConfigDir "$(router_windows_cli_path "$CLI_CONFIG_DIR")"
  [ -n "$CLI_CONFIG_FILE" ] && set -- "$@" -ConfigFile "$(router_windows_cli_path "$CLI_CONFIG_FILE")"
  [ -n "$CLI_DATABASE_URL" ] && set -- "$@" -DatabaseUrl "$(router_windows_database_url "$CLI_DATABASE_URL")"
  [ -n "$CLI_ROLES" ] && set -- "$@" -Roles "$CLI_ROLES"
  [ -n "$CLI_NODE_ID_PREFIX" ] && set -- "$@" -NodeIdPrefix "$CLI_NODE_ID_PREFIX"
  [ -n "$CLI_GATEWAY_BIND" ] && set -- "$@" -GatewayBind "$CLI_GATEWAY_BIND"
  [ -n "$CLI_ADMIN_BIND" ] && set -- "$@" -AdminBind "$CLI_ADMIN_BIND"
  [ -n "$CLI_PORTAL_BIND" ] && set -- "$@" -PortalBind "$CLI_PORTAL_BIND"
  [ -n "$CLI_GATEWAY_UPSTREAM" ] && set -- "$@" -GatewayUpstream "$CLI_GATEWAY_UPSTREAM"
  [ -n "$CLI_ADMIN_UPSTREAM" ] && set -- "$@" -AdminUpstream "$CLI_ADMIN_UPSTREAM"
  [ -n "$CLI_PORTAL_UPSTREAM" ] && set -- "$@" -PortalUpstream "$CLI_PORTAL_UPSTREAM"
  [ -n "$CLI_ADMIN_SITE_DIR" ] && set -- "$@" -AdminSiteDir "$(router_windows_cli_path "$CLI_ADMIN_SITE_DIR")"
  [ -n "$CLI_PORTAL_SITE_DIR" ] && set -- "$@" -PortalSiteDir "$(router_windows_cli_path "$CLI_PORTAL_SITE_DIR")"
  exec powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$PS_SCRIPT" "$@"
fi

if [ -z "$RUNTIME_HOME" ]; then
  if [ -f "$SCRIPT_DIR/$(router_binary_name router-product-service)" ]; then
    RUNTIME_HOME=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
  else
    RUNTIME_HOME="$DEFAULT_HOME"
  fi
fi

RUNTIME_HOME=$(router_resolve_absolute_path "$PWD" "$RUNTIME_HOME")
MANIFEST_FILE=$(router_release_manifest_path "$RUNTIME_HOME")
MANIFEST_INSTALL_MODE=$(router_release_manifest_string "$MANIFEST_FILE" 'installMode' || true)
MANIFEST_CONFIG_DIR=$(router_release_manifest_string "$MANIFEST_FILE" 'configRoot' || true)
MANIFEST_CONFIG_FILE=$(router_release_manifest_string "$MANIFEST_FILE" 'configFile' || true)
MANIFEST_DATA_DIR=$(router_release_manifest_string "$MANIFEST_FILE" 'mutableDataRoot' || true)
MANIFEST_LOG_DIR=$(router_release_manifest_string "$MANIFEST_FILE" 'logRoot' || true)
MANIFEST_RUN_DIR=$(router_release_manifest_string "$MANIFEST_FILE" 'runRoot' || true)

INSTALL_MODE=$(router_normalize_install_mode "${SDKWORK_ROUTER_INSTALL_MODE:-$MANIFEST_INSTALL_MODE}")
DEFAULT_CONFIG_DIR_RAW=$(router_default_config_root "$RUNTIME_HOME" "$INSTALL_MODE")
INITIAL_CONFIG_DIR="${SDKWORK_CONFIG_DIR:-}"
if [ -z "$INITIAL_CONFIG_DIR" ] && [ -n "${SDKWORK_CONFIG_FILE:-}" ]; then
  INITIAL_CONFIG_DIR=$(dirname -- "$SDKWORK_CONFIG_FILE")
fi
if [ -z "$INITIAL_CONFIG_DIR" ] && [ -n "$MANIFEST_CONFIG_DIR" ]; then
  INITIAL_CONFIG_DIR="$MANIFEST_CONFIG_DIR"
fi
if [ -z "$INITIAL_CONFIG_DIR" ]; then
  INITIAL_CONFIG_DIR="$DEFAULT_CONFIG_DIR_RAW"
fi
INITIAL_CONFIG_DIR=$(router_resolve_host_path "$INITIAL_CONFIG_DIR" "$DEFAULT_CONFIG_DIR_RAW")
ENV_FILE="$INITIAL_CONFIG_DIR/router.env"
router_load_env_file "$ENV_FILE"

INSTALL_MODE=$(router_normalize_install_mode "${SDKWORK_ROUTER_INSTALL_MODE:-$MANIFEST_INSTALL_MODE}")
BIN_DIR="$RUNTIME_HOME/bin"
BINARY_PATH="$BIN_DIR/$(router_binary_name router-product-service)"
DEFAULT_CONFIG_DIR_RAW=$(router_default_config_root "$RUNTIME_HOME" "$INSTALL_MODE")
DEFAULT_DATA_DIR_RAW=$(router_default_data_root "$RUNTIME_HOME" "$INSTALL_MODE")
DEFAULT_LOG_DIR_RAW=$(router_default_log_root "$RUNTIME_HOME" "$INSTALL_MODE")
DEFAULT_RUN_DIR_RAW=$(router_default_run_root "$RUNTIME_HOME" "$INSTALL_MODE")
DEFAULT_CONFIG_FILE_RAW="$DEFAULT_CONFIG_DIR_RAW/router.yaml"

CONFIG_DIR_RAW="${SDKWORK_CONFIG_DIR:-}"
if [ -z "$CONFIG_DIR_RAW" ] && [ -n "${SDKWORK_CONFIG_FILE:-}" ]; then
  CONFIG_DIR_RAW=$(dirname -- "$SDKWORK_CONFIG_FILE")
fi
if [ -z "$CONFIG_DIR_RAW" ] && [ -n "$MANIFEST_CONFIG_DIR" ]; then
  CONFIG_DIR_RAW="$MANIFEST_CONFIG_DIR"
fi
if [ -z "$CONFIG_DIR_RAW" ]; then
  CONFIG_DIR_RAW="$DEFAULT_CONFIG_DIR_RAW"
fi
CONFIG_DIR=$(router_resolve_host_path "$CONFIG_DIR_RAW" "$DEFAULT_CONFIG_DIR_RAW")

CONFIG_FILE_RAW="${SDKWORK_CONFIG_FILE:-}"
if [ -z "$CONFIG_FILE_RAW" ] && [ -n "$MANIFEST_CONFIG_FILE" ]; then
  CONFIG_FILE_RAW="$MANIFEST_CONFIG_FILE"
fi
if [ -z "$CONFIG_FILE_RAW" ]; then
  CONFIG_FILE_RAW="$DEFAULT_CONFIG_FILE_RAW"
fi
CONFIG_FILE_HOST=$(router_resolve_host_path "$CONFIG_FILE_RAW" "$DEFAULT_CONFIG_FILE_RAW")

DATA_DIR_RAW="$MANIFEST_DATA_DIR"
if [ -z "$DATA_DIR_RAW" ]; then
  DATA_DIR_RAW="$DEFAULT_DATA_DIR_RAW"
fi
DATA_DIR=$(router_resolve_host_path "$DATA_DIR_RAW" "$DEFAULT_DATA_DIR_RAW")

LOG_DIR_RAW="$MANIFEST_LOG_DIR"
if [ -z "$LOG_DIR_RAW" ]; then
  LOG_DIR_RAW="$DEFAULT_LOG_DIR_RAW"
fi
LOG_DIR=$(router_resolve_host_path "$LOG_DIR_RAW" "$DEFAULT_LOG_DIR_RAW")

RUN_DIR_RAW="$MANIFEST_RUN_DIR"
if [ -z "$RUN_DIR_RAW" ]; then
  RUN_DIR_RAW="$DEFAULT_RUN_DIR_RAW"
fi
RUN_DIR=$(router_resolve_host_path "$RUN_DIR_RAW" "$DEFAULT_RUN_DIR_RAW")

ENV_FILE="$CONFIG_DIR/router.env"
PID_FILE="$RUN_DIR/router-product-service.pid"
STATE_FILE="$RUN_DIR/router-product-service.state.env"
STDOUT_LOG="$LOG_DIR/router-product-service.stdout.log"
STDERR_LOG="$LOG_DIR/router-product-service.stderr.log"
PLAN_FILE="$RUN_DIR/router-product-service.plan.json"
DEFAULT_ADMIN_SITE_DIR="$RUNTIME_HOME/sites/admin/dist"
DEFAULT_PORTAL_SITE_DIR="$RUNTIME_HOME/sites/portal/dist"
DEFAULT_BOOTSTRAP_DATA_DIR="$RUNTIME_HOME/data"
REPOSITORY_BOOTSTRAP_DATA_DIR="$REPO_ROOT/data"
DEFAULT_CONFIG_DIR=$(router_portable_path "$CONFIG_DIR")
DEFAULT_CONFIG_FILE=$(router_portable_path "$CONFIG_FILE_HOST")
DEFAULT_DATABASE_URL=$(router_default_database_url "$DATA_DIR" "$INSTALL_MODE")
DEFAULT_ROUTER_BINARY="$BINARY_PATH"
DEFAULT_ADMIN_SITE_DIR_PORTABLE=$(router_portable_path "$DEFAULT_ADMIN_SITE_DIR")
DEFAULT_PORTAL_SITE_DIR_PORTABLE=$(router_portable_path "$DEFAULT_PORTAL_SITE_DIR")

router_ensure_dir "$CONFIG_DIR"
router_ensure_dir "$DATA_DIR"
router_ensure_dir "$LOG_DIR"
router_ensure_dir "$RUN_DIR"

SDKWORK_ROUTER_BINARY=${SDKWORK_ROUTER_BINARY:-"$DEFAULT_ROUTER_BINARY"}
SDKWORK_CONFIG_DIR=${SDKWORK_CONFIG_DIR:-"$DEFAULT_CONFIG_DIR"}
SDKWORK_CONFIG_FILE=${SDKWORK_CONFIG_FILE:-"$DEFAULT_CONFIG_FILE"}
SDKWORK_DATABASE_URL=${SDKWORK_DATABASE_URL:-"$DEFAULT_DATABASE_URL"}
SDKWORK_ROUTER_INSTALL_MODE=${SDKWORK_ROUTER_INSTALL_MODE:-"$INSTALL_MODE"}
SDKWORK_BOOTSTRAP_PROFILE=${SDKWORK_BOOTSTRAP_PROFILE:-"prod"}
if [ -z "${SDKWORK_BOOTSTRAP_DATA_DIR:-}" ]; then
  if [ -d "$DEFAULT_BOOTSTRAP_DATA_DIR" ]; then
    SDKWORK_BOOTSTRAP_DATA_DIR=$(router_portable_path "$DEFAULT_BOOTSTRAP_DATA_DIR")
  elif [ -d "$REPOSITORY_BOOTSTRAP_DATA_DIR" ]; then
    SDKWORK_BOOTSTRAP_DATA_DIR=$(router_portable_path "$REPOSITORY_BOOTSTRAP_DATA_DIR")
  fi
fi
SDKWORK_WEB_BIND=${SDKWORK_WEB_BIND:-"0.0.0.0:3001"}
SDKWORK_GATEWAY_BIND=${SDKWORK_GATEWAY_BIND:-"127.0.0.1:8080"}
SDKWORK_ADMIN_BIND=${SDKWORK_ADMIN_BIND:-"127.0.0.1:8081"}
SDKWORK_PORTAL_BIND=${SDKWORK_PORTAL_BIND:-"127.0.0.1:8082"}
SDKWORK_ADMIN_SITE_DIR=${SDKWORK_ADMIN_SITE_DIR:-"$DEFAULT_ADMIN_SITE_DIR_PORTABLE"}
SDKWORK_PORTAL_SITE_DIR=${SDKWORK_PORTAL_SITE_DIR:-"$DEFAULT_PORTAL_SITE_DIR_PORTABLE"}

[ -n "$CLI_BIND" ] && SDKWORK_WEB_BIND="$CLI_BIND"
[ -n "$CLI_CONFIG_DIR" ] && SDKWORK_CONFIG_DIR="$CLI_CONFIG_DIR"
[ -n "$CLI_CONFIG_FILE" ] && SDKWORK_CONFIG_FILE="$CLI_CONFIG_FILE"
[ -n "$CLI_DATABASE_URL" ] && SDKWORK_DATABASE_URL="$CLI_DATABASE_URL"
if [ -n "$CLI_ROLES" ]; then
  SDKWORK_ROUTER_ROLES="$CLI_ROLES"
fi
if [ -n "$CLI_NODE_ID_PREFIX" ]; then
  SDKWORK_ROUTER_NODE_ID_PREFIX="$CLI_NODE_ID_PREFIX"
fi
[ -n "$CLI_GATEWAY_BIND" ] && SDKWORK_GATEWAY_BIND="$CLI_GATEWAY_BIND"
[ -n "$CLI_ADMIN_BIND" ] && SDKWORK_ADMIN_BIND="$CLI_ADMIN_BIND"
[ -n "$CLI_PORTAL_BIND" ] && SDKWORK_PORTAL_BIND="$CLI_PORTAL_BIND"
if [ -n "$CLI_GATEWAY_UPSTREAM" ]; then
  SDKWORK_GATEWAY_PROXY_TARGET="$CLI_GATEWAY_UPSTREAM"
fi
if [ -n "$CLI_ADMIN_UPSTREAM" ]; then
  SDKWORK_ADMIN_PROXY_TARGET="$CLI_ADMIN_UPSTREAM"
fi
if [ -n "$CLI_PORTAL_UPSTREAM" ]; then
  SDKWORK_PORTAL_PROXY_TARGET="$CLI_PORTAL_UPSTREAM"
fi
[ -n "$CLI_ADMIN_SITE_DIR" ] && SDKWORK_ADMIN_SITE_DIR="$CLI_ADMIN_SITE_DIR"
[ -n "$CLI_PORTAL_SITE_DIR" ] && SDKWORK_PORTAL_SITE_DIR="$CLI_PORTAL_SITE_DIR"

SDKWORK_ROUTER_BINARY=$(router_resolve_host_path "$SDKWORK_ROUTER_BINARY" "$DEFAULT_ROUTER_BINARY")
SDKWORK_CONFIG_DIR=$(router_resolve_host_path "$SDKWORK_CONFIG_DIR" "$DEFAULT_CONFIG_DIR")
SDKWORK_CONFIG_FILE=$(router_resolve_host_path "$SDKWORK_CONFIG_FILE" "$DEFAULT_CONFIG_FILE")
SDKWORK_DATABASE_URL=$(router_resolve_host_database_url "$SDKWORK_DATABASE_URL" "$DEFAULT_DATABASE_URL")
SDKWORK_ADMIN_SITE_DIR=$(router_resolve_host_path "$SDKWORK_ADMIN_SITE_DIR" "$DEFAULT_ADMIN_SITE_DIR_PORTABLE")
SDKWORK_PORTAL_SITE_DIR=$(router_resolve_host_path "$SDKWORK_PORTAL_SITE_DIR" "$DEFAULT_PORTAL_SITE_DIR_PORTABLE")

export SDKWORK_ROUTER_BINARY
export SDKWORK_ROUTER_INSTALL_MODE
export SDKWORK_CONFIG_DIR
export SDKWORK_CONFIG_FILE
export SDKWORK_DATABASE_URL
export SDKWORK_BOOTSTRAP_PROFILE
[ -n "${SDKWORK_BOOTSTRAP_DATA_DIR:-}" ] && export SDKWORK_BOOTSTRAP_DATA_DIR
export SDKWORK_WEB_BIND
export SDKWORK_GATEWAY_BIND
export SDKWORK_ADMIN_BIND
export SDKWORK_PORTAL_BIND
export SDKWORK_ADMIN_SITE_DIR
export SDKWORK_PORTAL_SITE_DIR
[ -n "${SDKWORK_ROUTER_ROLES:-}" ] && export SDKWORK_ROUTER_ROLES || true
[ -n "${SDKWORK_ROUTER_NODE_ID_PREFIX:-}" ] && export SDKWORK_ROUTER_NODE_ID_PREFIX || true
[ -n "${SDKWORK_GATEWAY_PROXY_TARGET:-}" ] && export SDKWORK_GATEWAY_PROXY_TARGET || true
[ -n "${SDKWORK_ADMIN_PROXY_TARGET:-}" ] && export SDKWORK_ADMIN_PROXY_TARGET || true
[ -n "${SDKWORK_PORTAL_PROXY_TARGET:-}" ] && export SDKWORK_PORTAL_PROXY_TARGET || true

GATEWAY_HEALTH_URL=$(router_resolve_loopback_url "$SDKWORK_WEB_BIND" "/api/v1/health")
ADMIN_HEALTH_URL=$(router_resolve_loopback_url "$SDKWORK_WEB_BIND" "/api/admin/health")
PORTAL_HEALTH_URL=$(router_resolve_loopback_url "$SDKWORK_WEB_BIND" "/api/portal/health")

cd "$RUNTIME_HOME"
EXISTING_PID=$(router_get_running_pid "$PID_FILE" "$STATE_FILE")
if [ -n "$EXISTING_PID" ] && [ "$DRY_RUN" != '1' ]; then
  MANAGED_STATE_AVAILABLE=0
  if router_read_managed_state "$STATE_FILE"; then
    MANAGED_STATE_AVAILABLE=1
  fi

  ACTIVE_WEB_BIND="$SDKWORK_WEB_BIND"
  ACTIVE_GATEWAY_BIND="$SDKWORK_GATEWAY_BIND"
  ACTIVE_ADMIN_BIND="$SDKWORK_ADMIN_BIND"
  ACTIVE_PORTAL_BIND="$SDKWORK_PORTAL_BIND"
  if [ "$MANAGED_STATE_AVAILABLE" = '1' ]; then
    [ -n "${SDKWORK_ROUTER_MANAGED_WEB_BIND:-}" ] && ACTIVE_WEB_BIND="$SDKWORK_ROUTER_MANAGED_WEB_BIND"
    [ -n "${SDKWORK_ROUTER_MANAGED_GATEWAY_BIND:-}" ] && ACTIVE_GATEWAY_BIND="$SDKWORK_ROUTER_MANAGED_GATEWAY_BIND"
    [ -n "${SDKWORK_ROUTER_MANAGED_ADMIN_BIND:-}" ] && ACTIVE_ADMIN_BIND="$SDKWORK_ROUTER_MANAGED_ADMIN_BIND"
    [ -n "${SDKWORK_ROUTER_MANAGED_PORTAL_BIND:-}" ] && ACTIVE_PORTAL_BIND="$SDKWORK_ROUTER_MANAGED_PORTAL_BIND"
  fi

  ACTIVE_GATEWAY_HEALTH_URL=$(router_resolve_loopback_url "$ACTIVE_WEB_BIND" "/api/v1/health")
  ACTIVE_ADMIN_HEALTH_URL=$(router_resolve_loopback_url "$ACTIVE_WEB_BIND" "/api/admin/health")
  ACTIVE_PORTAL_HEALTH_URL=$(router_resolve_loopback_url "$ACTIVE_WEB_BIND" "/api/portal/health")

  if ! router_wait_for_url "$ACTIVE_GATEWAY_HEALTH_URL" "$WAIT_SECONDS" "$EXISTING_PID" \
    || ! router_wait_for_url "$ACTIVE_ADMIN_HEALTH_URL" "$WAIT_SECONDS" "$EXISTING_PID" \
    || ! router_wait_for_url "$ACTIVE_PORTAL_HEALTH_URL" "$WAIT_SECONDS" "$EXISTING_PID"; then
    if ! router_is_pid_running "$EXISTING_PID"; then
      rm -f "$PID_FILE"
      router_remove_managed_state "$STATE_FILE"
      router_log "previous production runtime pid=$EXISTING_PID exited during readiness checks; removed stale pid file and retrying startup"
    else
      router_log "production runtime pid=$EXISTING_PID is present but health checks are failing; recent logs follow"
      router_tail_log "$STDOUT_LOG"
      router_tail_log "$STDERR_LOG"
      router_die "production runtime already running (pid=$EXISTING_PID) but failed health checks"
    fi
  else
    if [ "$ACTIVE_WEB_BIND" != "$SDKWORK_WEB_BIND" ] \
      || [ "$ACTIVE_GATEWAY_BIND" != "$SDKWORK_GATEWAY_BIND" ] \
      || [ "$ACTIVE_ADMIN_BIND" != "$SDKWORK_ADMIN_BIND" ] \
      || [ "$ACTIVE_PORTAL_BIND" != "$SDKWORK_PORTAL_BIND" ]; then
      router_log "production runtime already running (pid=$EXISTING_PID) with active managed settings that differ from the requested launch configuration"
    else
      router_log "production runtime already running (pid=$EXISTING_PID)"
    fi
    router_startup_summary \
      'production release' \
      '1' \
      "$ACTIVE_WEB_BIND" \
      "$ACTIVE_GATEWAY_BIND" \
      "$ACTIVE_ADMIN_BIND" \
      "$ACTIVE_PORTAL_BIND" \
      '' \
      '' \
      "$STDOUT_LOG" \
      "$STDERR_LOG"
    exit 0
  fi
fi

if [ "$DRY_RUN" = '1' ]; then
  if [ -f "$SDKWORK_ROUTER_BINARY" ] && [ -d "$SDKWORK_ADMIN_SITE_DIR" ] && [ -d "$SDKWORK_PORTAL_SITE_DIR" ]; then
    "$SDKWORK_ROUTER_BINARY" --dry-run --plan-format json > "$PLAN_FILE"
  else
    router_render_release_dry_run_plan_json \
      "$SDKWORK_CONFIG_DIR" \
      "$SDKWORK_DATABASE_URL" \
      "$SDKWORK_WEB_BIND" \
      "$SDKWORK_GATEWAY_BIND" \
      "$SDKWORK_ADMIN_BIND" \
      "$SDKWORK_PORTAL_BIND" \
      "${SDKWORK_CONFIG_FILE:-}" \
      "${SDKWORK_ROUTER_ROLES:-}" \
      "${SDKWORK_ROUTER_NODE_ID_PREFIX:-}" \
      "${SDKWORK_GATEWAY_PROXY_TARGET:-}" \
      "${SDKWORK_ADMIN_PROXY_TARGET:-}" \
      "${SDKWORK_PORTAL_PROXY_TARGET:-}" \
      "$SDKWORK_ADMIN_SITE_DIR" \
      "$SDKWORK_PORTAL_SITE_DIR" > "$PLAN_FILE"
  fi
  cat "$PLAN_FILE"
  exit 0
fi

router_assert_bind_addresses_available \
  "production runtime" \
  "$SDKWORK_WEB_BIND" \
  "$SDKWORK_GATEWAY_BIND" \
  "$SDKWORK_ADMIN_BIND" \
  "$SDKWORK_PORTAL_BIND"

router_validate_file "router-product-service binary" "$SDKWORK_ROUTER_BINARY"
router_validate_dir "admin site directory" "$SDKWORK_ADMIN_SITE_DIR"
router_validate_dir "portal site directory" "$SDKWORK_PORTAL_SITE_DIR"

"$SDKWORK_ROUTER_BINARY" --dry-run --plan-format json > "$PLAN_FILE"

if [ "$FOREGROUND" = '1' ]; then
  exec "$SDKWORK_ROUTER_BINARY"
fi

router_warn_wsl_background_session "production runtime"

: > "$STDOUT_LOG"
: > "$STDERR_LOG"

PID=$(router_start_background_process "$SDKWORK_ROUTER_BINARY" "$RUNTIME_HOME" "$STDOUT_LOG" "$STDERR_LOG")
printf '%s\n' "$PID" > "$PID_FILE"
router_remove_managed_state "$STATE_FILE"

if ! router_wait_for_url "$GATEWAY_HEALTH_URL" "$WAIT_SECONDS" "$PID" \
  || ! router_wait_for_url "$ADMIN_HEALTH_URL" "$WAIT_SECONDS" "$PID" \
  || ! router_wait_for_url "$PORTAL_HEALTH_URL" "$WAIT_SECONDS" "$PID"; then
  RUNTIME_EXITED=0
  if ! router_is_pid_running "$PID"; then
    RUNTIME_EXITED=1
  fi
  router_tail_log "$STDOUT_LOG"
  router_tail_log "$STDERR_LOG"
  router_stop_pid "$PID" "$WAIT_SECONDS" 1 || true
  rm -f "$PID_FILE"
  router_remove_managed_state "$STATE_FILE"
  if [ "$RUNTIME_EXITED" = '1' ]; then
    router_die "production runtime exited before health checks completed; see startup log above"
  fi
  router_die "router-product-service failed health checks on $SDKWORK_WEB_BIND"
fi

if ! router_confirm_pid_alive "$PID" 2; then
  rm -f "$PID_FILE"
  router_remove_managed_state "$STATE_FILE"
  router_tail_log "$STDOUT_LOG"
  router_tail_log "$STDERR_LOG"
  router_die "production runtime exited immediately after reporting ready; see startup log above"
fi

PROCESS_FINGERPRINT=$(router_get_process_fingerprint "$PID" || true)
router_write_managed_state "$STATE_FILE" "$PID" "$PROCESS_FINGERPRINT" 'production release' "$SDKWORK_WEB_BIND" "$SDKWORK_GATEWAY_BIND" "$SDKWORK_ADMIN_BIND" "$SDKWORK_PORTAL_BIND" '1' '' ''

router_log "started router-product-service (pid=$PID)"
router_startup_summary \
  'production release' \
  '1' \
  "$SDKWORK_WEB_BIND" \
  "$SDKWORK_GATEWAY_BIND" \
  "$SDKWORK_ADMIN_BIND" \
  "$SDKWORK_PORTAL_BIND" \
  '' \
  '' \
  "$STDOUT_LOG" \
  "$STDERR_LOG"
