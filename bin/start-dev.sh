#!/usr/bin/env sh

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
# shellcheck source=bin/lib/runtime-common.sh
. "$SCRIPT_DIR/lib/runtime-common.sh"

REPO_ROOT=$(router_repo_root "$SCRIPT_DIR")
DEV_HOME=$(router_default_dev_home "$REPO_ROOT")
FOREGROUND=0
DRY_RUN=0
WAIT_SECONDS=600
WAIT_SECONDS_OVERRIDDEN=0
INSTALL_DEPS=0
PREVIEW_MODE=1
PROXY_DEV_MODE=0
TAURI_MODE=0

CLI_DATABASE_URL=''
CLI_GATEWAY_BIND=''
CLI_ADMIN_BIND=''
CLI_PORTAL_BIND=''
CLI_WEB_BIND=''

while [ "$#" -gt 0 ]; do
  case "$1" in
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
      WAIT_SECONDS_OVERRIDDEN=1
      shift 2
      ;;
    --install)
      INSTALL_DEPS=1
      shift
      ;;
    --preview)
      PREVIEW_MODE=1
      PROXY_DEV_MODE=0
      TAURI_MODE=0
      shift
      ;;
    --browser)
      PREVIEW_MODE=0
      PROXY_DEV_MODE=0
      TAURI_MODE=0
      shift
      ;;
    --proxy-dev)
      PREVIEW_MODE=0
      PROXY_DEV_MODE=1
      TAURI_MODE=0
      shift
      ;;
    --tauri)
      PREVIEW_MODE=0
      PROXY_DEV_MODE=0
      TAURI_MODE=1
      shift
      ;;
    --database-url)
      [ "$#" -ge 2 ] || router_die "--database-url requires a value"
      CLI_DATABASE_URL="$2"
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
    --web-bind)
      [ "$#" -ge 2 ] || router_die "--web-bind requires a value"
      CLI_WEB_BIND="$2"
      shift 2
      ;;
    *)
      router_die "unknown option: $1"
      ;;
    esac
done

if router_is_windows; then
  PS_SCRIPT="$(router_windows_path "$SCRIPT_DIR/start-dev.ps1")"
  set --
  [ "$FOREGROUND" = '1' ] && set -- "$@" -Foreground
  [ "$DRY_RUN" = '1' ] && set -- "$@" -DryRun
  [ "$WAIT_SECONDS" != '600' ] && set -- "$@" -WaitSeconds "$WAIT_SECONDS"
  [ "$INSTALL_DEPS" = '1' ] && set -- "$@" -Install
  [ "$PREVIEW_MODE" = '1' ] && set -- "$@" -Preview
  [ "$PROXY_DEV_MODE" = '1' ] && set -- "$@" -ProxyDev
  [ "$TAURI_MODE" = '1' ] && set -- "$@" -Tauri
  [ "$PREVIEW_MODE" = '0' ] && [ "$PROXY_DEV_MODE" = '0' ] && [ "$TAURI_MODE" = '0' ] && set -- "$@" -Browser
  [ -n "$CLI_DATABASE_URL" ] && set -- "$@" -DatabaseUrl "$(router_windows_database_url "$CLI_DATABASE_URL")"
  [ -n "$CLI_GATEWAY_BIND" ] && set -- "$@" -GatewayBind "$CLI_GATEWAY_BIND"
  [ -n "$CLI_ADMIN_BIND" ] && set -- "$@" -AdminBind "$CLI_ADMIN_BIND"
  [ -n "$CLI_PORTAL_BIND" ] && set -- "$@" -PortalBind "$CLI_PORTAL_BIND"
  [ -n "$CLI_WEB_BIND" ] && set -- "$@" -WebBind "$CLI_WEB_BIND"
  exec powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$PS_SCRIPT" "$@"
fi

if [ "$WAIT_SECONDS_OVERRIDDEN" != '1' ] \
  && router_is_wsl \
  && router_is_wsl_windows_mount_path "$REPO_ROOT"; then
  WAIT_SECONDS=1800
  router_log "WSL launch from a Windows-mounted worktree detected; extending readiness timeout to ${WAIT_SECONDS} seconds to accommodate frontend reinstalls."
fi

CONFIG_DIR="$DEV_HOME/config"
DATA_DIR="$DEV_HOME/data"
LOG_DIR="$DEV_HOME/log"
RUN_DIR="$DEV_HOME/run"
ENV_FILE="$CONFIG_DIR/router-dev.env"
PID_FILE="$RUN_DIR/start-workspace.pid"
STOP_FILE="$RUN_DIR/start-workspace.stop"
STATE_FILE="$RUN_DIR/start-workspace.state.env"
STDOUT_LOG="$LOG_DIR/start-workspace.stdout.log"
STDERR_LOG="$LOG_DIR/start-workspace.stderr.log"
PLAN_FILE="$RUN_DIR/start-workspace.plan.txt"

router_ensure_dir "$CONFIG_DIR"
router_ensure_dir "$DATA_DIR"
router_ensure_dir "$LOG_DIR"
router_ensure_dir "$RUN_DIR"

router_load_env_file "$ENV_FILE"

BOOTSTRAP_DATA_DIR="$REPO_ROOT/data"
SDKWORK_DATABASE_URL=${SDKWORK_DATABASE_URL:-"sqlite://$(router_portable_path "$DATA_DIR")/sdkwork-api-router-dev.db"}
SDKWORK_BOOTSTRAP_PROFILE=${SDKWORK_BOOTSTRAP_PROFILE:-"dev"}
if [ -z "${SDKWORK_BOOTSTRAP_DATA_DIR:-}" ] && [ -d "$BOOTSTRAP_DATA_DIR" ]; then
  SDKWORK_BOOTSTRAP_DATA_DIR=$(router_portable_path "$BOOTSTRAP_DATA_DIR")
fi
SDKWORK_GATEWAY_BIND=${SDKWORK_GATEWAY_BIND:-"127.0.0.1:9980"}
SDKWORK_ADMIN_BIND=${SDKWORK_ADMIN_BIND:-"127.0.0.1:9981"}
SDKWORK_PORTAL_BIND=${SDKWORK_PORTAL_BIND:-"127.0.0.1:9982"}
SDKWORK_WEB_BIND=${SDKWORK_WEB_BIND:-"127.0.0.1:9983"}

export SDKWORK_BOOTSTRAP_PROFILE
[ -n "${SDKWORK_BOOTSTRAP_DATA_DIR:-}" ] && export SDKWORK_BOOTSTRAP_DATA_DIR

[ -n "$CLI_DATABASE_URL" ] && SDKWORK_DATABASE_URL="$CLI_DATABASE_URL"
[ -n "$CLI_GATEWAY_BIND" ] && SDKWORK_GATEWAY_BIND="$CLI_GATEWAY_BIND"
[ -n "$CLI_ADMIN_BIND" ] && SDKWORK_ADMIN_BIND="$CLI_ADMIN_BIND"
[ -n "$CLI_PORTAL_BIND" ] && SDKWORK_PORTAL_BIND="$CLI_PORTAL_BIND"
[ -n "$CLI_WEB_BIND" ] && SDKWORK_WEB_BIND="$CLI_WEB_BIND"

if [ ! -d "$REPO_ROOT/apps/sdkwork-router-admin/node_modules" ] || [ ! -d "$REPO_ROOT/apps/sdkwork-router-portal/node_modules" ]; then
  INSTALL_DEPS=1
fi

router_validate_file "workspace launcher" "$REPO_ROOT/scripts/dev/start-workspace.mjs"

set -- \
  scripts/dev/start-workspace.mjs \
  --database-url "$SDKWORK_DATABASE_URL" \
  --gateway-bind "$SDKWORK_GATEWAY_BIND" \
  --admin-bind "$SDKWORK_ADMIN_BIND" \
  --portal-bind "$SDKWORK_PORTAL_BIND" \
  --web-bind "$SDKWORK_WEB_BIND" \
  --stop-file "$STOP_FILE"

[ "$INSTALL_DEPS" = '1' ] && set -- "$@" --install
[ "$PREVIEW_MODE" = '1' ] && set -- "$@" --preview
[ "$PROXY_DEV_MODE" = '1' ] && set -- "$@" --proxy-dev
[ "$TAURI_MODE" = '1' ] && set -- "$@" --tauri

GATEWAY_HEALTH_URL=$(router_resolve_loopback_url "$SDKWORK_GATEWAY_BIND" "/health")
ADMIN_HEALTH_URL=$(router_resolve_loopback_url "$SDKWORK_ADMIN_BIND" "/admin/health")
PORTAL_HEALTH_URL=$(router_resolve_loopback_url "$SDKWORK_PORTAL_BIND" "/portal/health")
PREVIEW_ADMIN_URL=$(router_resolve_loopback_url "$SDKWORK_WEB_BIND" "/admin/")
PREVIEW_PORTAL_URL=$(router_resolve_loopback_url "$SDKWORK_WEB_BIND" "/portal/")
BROWSER_ADMIN_URL='http://127.0.0.1:5173/admin/'
BROWSER_PORTAL_URL='http://127.0.0.1:5174/portal/'

if [ "$PREVIEW_MODE" = '1' ] || [ "$PROXY_DEV_MODE" = '1' ] || [ "$TAURI_MODE" = '1' ]; then
  PRIMARY_ADMIN_URL="$PREVIEW_ADMIN_URL"
  PRIMARY_PORTAL_URL="$PREVIEW_PORTAL_URL"
  PRIMARY_MODE='development preview'
  [ "$PROXY_DEV_MODE" = '1' ] && PRIMARY_MODE='development proxy hot reload'
  [ "$TAURI_MODE" = '1' ] && PRIMARY_MODE='development tauri'
  PRIMARY_UNIFIED_ACCESS_ENABLED='1'
  SECONDARY_ADMIN_URL="$BROWSER_ADMIN_URL"
  SECONDARY_PORTAL_URL="$BROWSER_PORTAL_URL"
  SECONDARY_MODE='development browser'
  SECONDARY_UNIFIED_ACCESS_ENABLED='0'
else
  PRIMARY_ADMIN_URL="$BROWSER_ADMIN_URL"
  PRIMARY_PORTAL_URL="$BROWSER_PORTAL_URL"
  PRIMARY_MODE='development browser'
  PRIMARY_UNIFIED_ACCESS_ENABLED='0'
  SECONDARY_ADMIN_URL="$PREVIEW_ADMIN_URL"
  SECONDARY_PORTAL_URL="$PREVIEW_PORTAL_URL"
  SECONDARY_MODE='development preview'
  SECONDARY_UNIFIED_ACCESS_ENABLED='1'
fi

cd "$REPO_ROOT"
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

  ACTIVE_GATEWAY_HEALTH_URL=$(router_resolve_loopback_url "$ACTIVE_GATEWAY_BIND" "/health")
  ACTIVE_ADMIN_HEALTH_URL=$(router_resolve_loopback_url "$ACTIVE_ADMIN_BIND" "/admin/health")
  ACTIVE_PORTAL_HEALTH_URL=$(router_resolve_loopback_url "$ACTIVE_PORTAL_BIND" "/portal/health")

  if ! router_wait_for_url "$ACTIVE_GATEWAY_HEALTH_URL" "$WAIT_SECONDS" "$EXISTING_PID" \
    || ! router_wait_for_url "$ACTIVE_ADMIN_HEALTH_URL" "$WAIT_SECONDS" "$EXISTING_PID" \
    || ! router_wait_for_url "$ACTIVE_PORTAL_HEALTH_URL" "$WAIT_SECONDS" "$EXISTING_PID"; then
    if ! router_is_pid_running "$EXISTING_PID"; then
      rm -f "$PID_FILE" "$STOP_FILE"
      router_remove_managed_state "$STATE_FILE"
      router_log "previous development workspace pid=$EXISTING_PID exited during readiness checks; removed stale pid file and retrying startup"
    else
      router_log "development workspace pid=$EXISTING_PID is present but managed services are not healthy; recent logs follow"
      router_tail_log "$STDOUT_LOG"
      router_tail_log "$STDERR_LOG"
      router_die "development workspace already running (pid=$EXISTING_PID) but failed health checks"
    fi
  else
    DEV_ADMIN_URL="$PRIMARY_ADMIN_URL"
    DEV_PORTAL_URL="$PRIMARY_PORTAL_URL"
    ROUTER_MODE="$PRIMARY_MODE"
    UNIFIED_ACCESS_ENABLED="$PRIMARY_UNIFIED_ACCESS_ENABLED"

    if [ "$MANAGED_STATE_AVAILABLE" = '1' ]; then
      [ -n "${SDKWORK_ROUTER_MANAGED_ADMIN_APP_URL:-}" ] && DEV_ADMIN_URL="$SDKWORK_ROUTER_MANAGED_ADMIN_APP_URL"
      [ -n "${SDKWORK_ROUTER_MANAGED_PORTAL_APP_URL:-}" ] && DEV_PORTAL_URL="$SDKWORK_ROUTER_MANAGED_PORTAL_APP_URL"
      [ -n "${SDKWORK_ROUTER_MANAGED_MODE:-}" ] && ROUTER_MODE="$SDKWORK_ROUTER_MANAGED_MODE"
      [ -n "${SDKWORK_ROUTER_MANAGED_UNIFIED_ACCESS_ENABLED:-}" ] && UNIFIED_ACCESS_ENABLED="$SDKWORK_ROUTER_MANAGED_UNIFIED_ACCESS_ENABLED"
    fi

    if ! router_wait_for_url "$DEV_ADMIN_URL" "$WAIT_SECONDS" "$EXISTING_PID" \
      || ! router_wait_for_url "$DEV_PORTAL_URL" "$WAIT_SECONDS" "$EXISTING_PID"; then
      if [ "$MANAGED_STATE_AVAILABLE" = '1' ]; then
        if ! router_is_pid_running "$EXISTING_PID"; then
          rm -f "$PID_FILE" "$STOP_FILE"
          router_remove_managed_state "$STATE_FILE"
          router_log "previous development workspace pid=$EXISTING_PID exited during readiness checks; removed stale pid file and retrying startup"
        else
          router_log "development workspace pid=$EXISTING_PID is present but web surfaces are not healthy; recent logs follow"
          router_tail_log "$STDOUT_LOG"
          router_tail_log "$STDERR_LOG"
          router_die "development workspace already running (pid=$EXISTING_PID) but failed health checks"
        fi
      else
      DEV_ADMIN_URL="$SECONDARY_ADMIN_URL"
      DEV_PORTAL_URL="$SECONDARY_PORTAL_URL"
      ROUTER_MODE="$SECONDARY_MODE"
      UNIFIED_ACCESS_ENABLED="$SECONDARY_UNIFIED_ACCESS_ENABLED"
      if ! router_wait_for_url "$DEV_ADMIN_URL" "$WAIT_SECONDS" "$EXISTING_PID" \
        || ! router_wait_for_url "$DEV_PORTAL_URL" "$WAIT_SECONDS" "$EXISTING_PID"; then
        if ! router_is_pid_running "$EXISTING_PID"; then
          rm -f "$PID_FILE" "$STOP_FILE"
          router_remove_managed_state "$STATE_FILE"
          router_log "previous development workspace pid=$EXISTING_PID exited during readiness checks; removed stale pid file and retrying startup"
        else
          router_log "development workspace pid=$EXISTING_PID is present but web surfaces are not healthy; recent logs follow"
          router_tail_log "$STDOUT_LOG"
          router_tail_log "$STDERR_LOG"
          router_die "development workspace already running (pid=$EXISTING_PID) but failed health checks"
        fi
      else
        router_log "development workspace already running (pid=$EXISTING_PID)"
        router_startup_summary \
          "$ROUTER_MODE" \
          "$UNIFIED_ACCESS_ENABLED" \
          "$ACTIVE_WEB_BIND" \
          "$ACTIVE_GATEWAY_BIND" \
          "$ACTIVE_ADMIN_BIND" \
          "$ACTIVE_PORTAL_BIND" \
          "$DEV_ADMIN_URL" \
          "$DEV_PORTAL_URL" \
          "$STDOUT_LOG" \
          "$STDERR_LOG"
        exit 0
      fi
      fi
    else
      if [ "$ACTIVE_WEB_BIND" != "$SDKWORK_WEB_BIND" ] \
        || [ "$ACTIVE_GATEWAY_BIND" != "$SDKWORK_GATEWAY_BIND" ] \
        || [ "$ACTIVE_ADMIN_BIND" != "$SDKWORK_ADMIN_BIND" ] \
        || [ "$ACTIVE_PORTAL_BIND" != "$SDKWORK_PORTAL_BIND" ] \
        || [ "$ROUTER_MODE" != "$PRIMARY_MODE" ]; then
        router_log "development workspace already running (pid=$EXISTING_PID) with active managed settings that differ from the requested launch configuration"
      else
        router_log "development workspace already running (pid=$EXISTING_PID)"
      fi
      router_startup_summary \
        "$ROUTER_MODE" \
        "$UNIFIED_ACCESS_ENABLED" \
        "$ACTIVE_WEB_BIND" \
        "$ACTIVE_GATEWAY_BIND" \
        "$ACTIVE_ADMIN_BIND" \
        "$ACTIVE_PORTAL_BIND" \
        "$DEV_ADMIN_URL" \
        "$DEV_PORTAL_URL" \
        "$STDOUT_LOG" \
        "$STDERR_LOG"
      exit 0
    fi
  fi
fi

node "$@" --dry-run > "$PLAN_FILE"

if [ "$DRY_RUN" = '1' ]; then
  cat "$PLAN_FILE"
  exit 0
fi

if [ "$PREVIEW_MODE" = '1' ] || [ "$TAURI_MODE" = '1' ]; then
  router_assert_bind_addresses_available \
    "development workspace" \
    "$SDKWORK_GATEWAY_BIND" \
    "$SDKWORK_ADMIN_BIND" \
    "$SDKWORK_PORTAL_BIND" \
    "$SDKWORK_WEB_BIND"
elif [ "$PROXY_DEV_MODE" = '1' ]; then
  router_assert_bind_addresses_available \
    "development workspace" \
    "$SDKWORK_GATEWAY_BIND" \
    "$SDKWORK_ADMIN_BIND" \
    "$SDKWORK_PORTAL_BIND" \
    "$SDKWORK_WEB_BIND" \
    '127.0.0.1:5173' \
    '127.0.0.1:5174'
else
  router_assert_bind_addresses_available \
    "development workspace" \
    "$SDKWORK_GATEWAY_BIND" \
    "$SDKWORK_ADMIN_BIND" \
    "$SDKWORK_PORTAL_BIND" \
    '127.0.0.1:5173' \
    '127.0.0.1:5174'
fi

if [ "$FOREGROUND" = '1' ]; then
  rm -f "$STOP_FILE"
  exec node "$@"
fi

router_warn_wsl_background_session "development workspace"

rm -f "$STOP_FILE"
: > "$STDOUT_LOG"
: > "$STDERR_LOG"

NODE_BIN=$(command -v node 2>/dev/null || printf '%s' node)
PID=$(router_start_background_process "$NODE_BIN" "$REPO_ROOT" "$STDOUT_LOG" "$STDERR_LOG" "$@")
printf '%s\n' "$PID" > "$PID_FILE"
router_remove_managed_state "$STATE_FILE"

if ! router_wait_for_url "$GATEWAY_HEALTH_URL" "$WAIT_SECONDS" "$PID" \
  || ! router_wait_for_url "$ADMIN_HEALTH_URL" "$WAIT_SECONDS" "$PID" \
  || ! router_wait_for_url "$PORTAL_HEALTH_URL" "$WAIT_SECONDS" "$PID"; then
  WORKSPACE_EXITED=0
  if ! router_is_pid_running "$PID"; then
    WORKSPACE_EXITED=1
  fi
  router_tail_log "$STDOUT_LOG"
  router_tail_log "$STDERR_LOG"
  router_stop_pid "$PID" "$WAIT_SECONDS" 1 || true
  rm -f "$PID_FILE"
  rm -f "$STOP_FILE"
  router_remove_managed_state "$STATE_FILE"
  if [ "$WORKSPACE_EXITED" = '1' ]; then
    router_die "development workspace exited before backend health checks completed; see startup log above"
  fi
  router_die "development services failed health checks"
fi

DEV_ADMIN_URL="$PRIMARY_ADMIN_URL"
DEV_PORTAL_URL="$PRIMARY_PORTAL_URL"

if ! router_wait_for_url "$DEV_ADMIN_URL" "$WAIT_SECONDS" "$PID" \
  || ! router_wait_for_url "$DEV_PORTAL_URL" "$WAIT_SECONDS" "$PID"; then
  WORKSPACE_EXITED=0
  if ! router_is_pid_running "$PID"; then
    WORKSPACE_EXITED=1
  fi
  router_tail_log "$STDOUT_LOG"
  router_tail_log "$STDERR_LOG"
  router_stop_pid "$PID" "$WAIT_SECONDS" 1 || true
  rm -f "$PID_FILE"
  rm -f "$STOP_FILE"
  router_remove_managed_state "$STATE_FILE"
  if [ "$WORKSPACE_EXITED" = '1' ]; then
    router_die "development workspace exited before web surfaces became ready; see startup log above"
  fi
  router_die "development web surfaces failed health checks"
fi

if ! router_confirm_pid_alive "$PID" 2; then
  rm -f "$PID_FILE"
  rm -f "$STOP_FILE"
  router_remove_managed_state "$STATE_FILE"
  router_tail_log "$STDOUT_LOG"
  router_tail_log "$STDERR_LOG"
  router_die "development workspace exited immediately after reporting ready; see startup log above"
fi

router_log "started development workspace (pid=$PID)"
if [ "$PREVIEW_MODE" = '1' ]; then
  ROUTER_MODE='development preview'
  UNIFIED_ACCESS_ENABLED='1'
elif [ "$PROXY_DEV_MODE" = '1' ]; then
  ROUTER_MODE='development proxy hot reload'
  UNIFIED_ACCESS_ENABLED='1'
elif [ "$TAURI_MODE" = '1' ]; then
  ROUTER_MODE='development tauri'
  UNIFIED_ACCESS_ENABLED='1'
else
  ROUTER_MODE='development browser'
  UNIFIED_ACCESS_ENABLED='0'
fi

PROCESS_FINGERPRINT=$(router_get_process_fingerprint "$PID" || true)
router_write_managed_state "$STATE_FILE" "$PID" "$PROCESS_FINGERPRINT" "$ROUTER_MODE" "$SDKWORK_WEB_BIND" "$SDKWORK_GATEWAY_BIND" "$SDKWORK_ADMIN_BIND" "$SDKWORK_PORTAL_BIND" "$UNIFIED_ACCESS_ENABLED" "$DEV_ADMIN_URL" "$DEV_PORTAL_URL"

router_startup_summary \
  "$ROUTER_MODE" \
  "$UNIFIED_ACCESS_ENABLED" \
  "$SDKWORK_WEB_BIND" \
  "$SDKWORK_GATEWAY_BIND" \
  "$SDKWORK_ADMIN_BIND" \
  "$SDKWORK_PORTAL_BIND" \
  "$DEV_ADMIN_URL" \
  "$DEV_PORTAL_URL" \
  "$STDOUT_LOG" \
  "$STDERR_LOG"
