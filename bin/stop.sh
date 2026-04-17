#!/usr/bin/env sh

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
# shellcheck source=bin/lib/runtime-common.sh
. "$SCRIPT_DIR/lib/runtime-common.sh"

REPO_ROOT=$(router_repo_root "$SCRIPT_DIR")
DEFAULT_HOME=$(router_default_install_home "$REPO_ROOT")

RUNTIME_HOME=''
DRY_RUN=0
WAIT_SECONDS=30
FORCE_MODE=1

while [ "$#" -gt 0 ]; do
  case "$1" in
    --home)
      [ "$#" -ge 2 ] || router_die "--home requires a value"
      RUNTIME_HOME="$2"
      shift 2
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
    --graceful-only)
      FORCE_MODE=0
      shift
      ;;
    *)
      router_die "unknown option: $1"
      ;;
  esac
done

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
MANIFEST_LOG_DIR=$(router_release_manifest_string "$MANIFEST_FILE" 'logRoot' || true)
MANIFEST_RUN_DIR=$(router_release_manifest_string "$MANIFEST_FILE" 'runRoot' || true)

INSTALL_MODE=$(router_normalize_install_mode "${SDKWORK_ROUTER_INSTALL_MODE:-$MANIFEST_INSTALL_MODE}")
DEFAULT_CONFIG_DIR_RAW=$(router_default_config_root "$RUNTIME_HOME" "$INSTALL_MODE")
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
ENV_FILE="$CONFIG_DIR/router.env"
router_load_env_file "$ENV_FILE"

INSTALL_MODE=$(router_normalize_install_mode "${SDKWORK_ROUTER_INSTALL_MODE:-$MANIFEST_INSTALL_MODE}")
DEFAULT_LOG_DIR_RAW=$(router_default_log_root "$RUNTIME_HOME" "$INSTALL_MODE")
DEFAULT_RUN_DIR_RAW=$(router_default_run_root "$RUNTIME_HOME" "$INSTALL_MODE")
LOG_DIR_RAW="$MANIFEST_LOG_DIR"
if [ -z "$LOG_DIR_RAW" ]; then
  LOG_DIR_RAW="$DEFAULT_LOG_DIR_RAW"
fi
RUN_DIR_RAW="$MANIFEST_RUN_DIR"
if [ -z "$RUN_DIR_RAW" ]; then
  RUN_DIR_RAW="$DEFAULT_RUN_DIR_RAW"
fi
LOG_DIR=$(router_resolve_host_path "$LOG_DIR_RAW" "$DEFAULT_LOG_DIR_RAW")
RUN_DIR=$(router_resolve_host_path "$RUN_DIR_RAW" "$DEFAULT_RUN_DIR_RAW")

if router_is_windows; then
  PS_SCRIPT="$(router_windows_path "$SCRIPT_DIR/stop.ps1")"
  set --
  [ -n "$RUNTIME_HOME" ] && set -- "$@" -Home "$(router_windows_path "$RUNTIME_HOME")"
  [ "$DRY_RUN" = '1' ] && set -- "$@" -DryRun
  [ "$WAIT_SECONDS" != '30' ] && set -- "$@" -WaitSeconds "$WAIT_SECONDS"
  [ "$FORCE_MODE" = '0' ] && set -- "$@" -GracefulOnly
  exec powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$PS_SCRIPT" "$@"
fi

PID_FILE="$RUN_DIR/router-product-service.pid"
STATE_FILE="$RUN_DIR/router-product-service.state.env"
STDOUT_LOG="$LOG_DIR/router-product-service.stdout.log"
STDERR_LOG="$LOG_DIR/router-product-service.stderr.log"

if [ "$DRY_RUN" = '1' ]; then
  router_log "would stop router-product-service using pid file $PID_FILE"
  exit 0
fi

if ! [ -f "$PID_FILE" ]; then
  router_remove_managed_state "$STATE_FILE"
  router_log "pid file not found, nothing to stop: $PID_FILE"
  exit 0
fi

PID=$(router_get_running_pid "$PID_FILE" "$STATE_FILE")
if [ -z "$PID" ]; then
  rm -f "$PID_FILE"
  router_remove_managed_state "$STATE_FILE"
  router_log "process already stopped, removed stale pid file: $PID_FILE"
  exit 0
fi

if ! router_stop_pid "$PID" "$WAIT_SECONDS" "$FORCE_MODE"; then
  router_tail_log "$STDOUT_LOG"
  router_tail_log "$STDERR_LOG"
  router_die "failed to stop router-product-service pid=$PID"
fi

rm -f "$PID_FILE"
router_remove_managed_state "$STATE_FILE"
router_log "stopped router-product-service pid=$PID"
