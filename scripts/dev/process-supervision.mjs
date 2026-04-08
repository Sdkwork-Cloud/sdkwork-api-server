import { spawn } from 'node:child_process';
import process from 'node:process';

function hasChildExited(child) {
  return child.exitCode !== null || child.signalCode !== null;
}

export function didChildExitFail(code, signal) {
  return Boolean(signal) || (code ?? 0) !== 0;
}

export function createSupervisorKeepAlive({
  intervalMs = 0x3fffffff,
  setIntervalImpl = setInterval,
  clearIntervalImpl = clearInterval,
} = {}) {
  const timer = setIntervalImpl(() => {}, intervalMs);
  let released = false;

  return () => {
    if (released) {
      return;
    }
    released = true;
    clearIntervalImpl(timer);
  };
}

function terminateWindowsProcessTree(child, forceSignal, spawnImpl) {
  if (!child?.pid) {
    return false;
  }

  const args = ['/PID', String(child.pid), '/T'];
  if (forceSignal) {
    args.push('/F');
  }

  try {
    const killer = spawnImpl('taskkill', args, {
      stdio: 'ignore',
      windowsHide: true,
    });
    if (killer && typeof killer.once === 'function') {
      killer.once('error', () => {});
    }
    return true;
  } catch {
    return false;
  }
}

function requestChildTermination(child, signal, { platform, spawnImpl }) {
  if (
    platform === 'win32'
    && terminateWindowsProcessTree(child, signal === 'SIGKILL', spawnImpl)
  ) {
    return;
  }

  try {
    child.kill(signal);
  } catch {
  }
}

export function waitForChildExit(
  child,
  {
    termSignal = 'SIGTERM',
    forceSignal = 'SIGKILL',
    forceKillAfterMs = 5000,
    settleAfterForceMs = 5000,
    setTimer = setTimeout,
    clearTimer = clearTimeout,
    platform = process.platform,
    spawnImpl = spawn,
  } = {},
) {
  if (!child || hasChildExited(child)) {
    return Promise.resolve();
  }

  return new Promise((resolve) => {
    let finished = false;
    let forceTimer = null;
    let settleTimer = null;

    const finish = () => {
      if (finished) {
        return;
      }
      finished = true;
      if (forceTimer) {
        clearTimer(forceTimer);
      }
      if (settleTimer) {
        clearTimer(settleTimer);
      }
      child.removeListener('exit', onExit);
      resolve();
    };

    const onExit = () => finish();
    child.once('exit', onExit);

    requestChildTermination(child, termSignal, { platform, spawnImpl });

    forceTimer = setTimer(() => {
      if (hasChildExited(child)) {
        finish();
        return;
      }

      requestChildTermination(child, forceSignal, { platform, spawnImpl });

      settleTimer = setTimer(() => finish(), settleAfterForceMs);
    }, forceKillAfterMs);
  });
}

export function createSignalController({
  label,
  children,
  logger = console.log,
  exit = (code) => process.exit(code),
  onShutdownStart = null,
  forceKillAfterMs = 5000,
  settleAfterForceMs = 5000,
} = {}) {
  if (typeof label !== 'string' || label.trim().length === 0) {
    throw new Error('label is required');
  }

  if (!Array.isArray(children)) {
    throw new Error('children must be an array');
  }

  let shutdownPromise = null;

  return {
    shutdown(signal, exitCode = 0) {
      if (shutdownPromise) {
        return shutdownPromise;
      }

      if (typeof onShutdownStart === 'function') {
        onShutdownStart();
      }

      logger(`[${label}] received ${signal}, stopping child processes`);
      const currentChildren = [...children].filter(Boolean);
      shutdownPromise = Promise.allSettled(
        currentChildren.map((child) => waitForChildExit(child, {
          forceKillAfterMs,
          settleAfterForceMs,
        })),
      ).then(() => {
        exit(exitCode);
      });

      return shutdownPromise;
    },

    register(processRef = process) {
      processRef.on('SIGINT', () => {
        void this.shutdown('SIGINT');
      });
      processRef.on('SIGTERM', () => {
        void this.shutdown('SIGTERM');
      });
    },
  };
}
