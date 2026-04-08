#!/usr/bin/env node

import path from 'node:path';
import process from 'node:process';

export function resolveWorkspaceTargetDir({
  workspaceRoot,
  env = process.env,
  platform = process.platform,
} = {}) {
  if (typeof workspaceRoot !== 'string' || workspaceRoot.trim().length === 0) {
    throw new Error('workspaceRoot is required.');
  }

  const requestedTargetDir = String(env.CARGO_TARGET_DIR ?? '').trim();
  if (requestedTargetDir.length > 0) {
    return path.isAbsolute(requestedTargetDir)
      ? requestedTargetDir
      : path.resolve(workspaceRoot, requestedTargetDir);
  }

  if (platform !== 'win32') {
    return path.join(workspaceRoot, 'target');
  }

  return path.join(workspaceRoot, 'bin', '.sdkwork-target-vs2022');
}

export function withManagedWorkspaceTargetDir({
  workspaceRoot,
  env = process.env,
  platform = process.platform,
} = {}) {
  const nextEnv = { ...env };
  if (platform === 'win32' && String(nextEnv.CARGO_TARGET_DIR ?? '').trim().length === 0) {
    nextEnv.CARGO_TARGET_DIR = resolveWorkspaceTargetDir({
      workspaceRoot,
      env: nextEnv,
      platform,
    });
  }

  return nextEnv;
}
