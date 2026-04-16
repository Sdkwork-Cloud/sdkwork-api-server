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
RUN_DIR="$RUNTIME_HOME/var/run"
LOG_DIR="$RUNTIME_HOME/var/log"

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
