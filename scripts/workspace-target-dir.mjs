#!/usr/bin/env node

import { createHash } from 'node:crypto';
import { existsSync, mkdirSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import process from 'node:process';

function resolveConfiguredManagedWindowsRoot(workspaceRoot, configuredRootValue = '') {
  const configuredRoot = String(configuredRootValue ?? '').trim();
  if (configuredRoot.length === 0) {
    return '';
  }

  const resolvedRoot = path.isAbsolute(configuredRoot)
    ? configuredRoot
    : path.resolve(workspaceRoot, configuredRoot);
  return existsSync(resolvedRoot) ? resolvedRoot : '';
}

function resolveConfiguredManagedWindowsTargetRoot(workspaceRoot, env = process.env) {
  return resolveConfiguredManagedWindowsRoot(workspaceRoot, env.SDKWORK_WINDOWS_TARGET_ROOT);
}

function resolveConfiguredManagedWindowsTempRoot(workspaceRoot, env = process.env) {
  return resolveConfiguredManagedWindowsRoot(
    workspaceRoot,
    String(env.SDKWORK_WINDOWS_TEMP_ROOT ?? '').trim()
      || String(env.SDKWORK_WINDOWS_TARGET_ROOT ?? '').trim(),
  );
}

function managedWindowsWorkspaceTargetLeaf(workspaceRoot) {
  const resolvedWorkspaceRoot = path.resolve(workspaceRoot);
  const normalizedWorkspaceRoot = resolvedWorkspaceRoot.replaceAll('\\', '/').toLowerCase();
  const workspaceName = path.basename(resolvedWorkspaceRoot)
    .toLowerCase()
    .replaceAll(/[^a-z0-9]+/g, '')
    .slice(0, 12) || 'workspace';
  const workspaceHash = createHash('sha1')
    .update(normalizedWorkspaceRoot)
    .digest('hex')
    .slice(0, 10);

  return `${workspaceName}-${workspaceHash}`;
}

function defaultManagedWindowsWorkspaceRoot(
  workspaceRoot,
  directoryName,
  hostPlatform = process.platform,
) {
  const resolvedWorkspaceRoot = path.resolve(workspaceRoot);
  const workspaceDriveRoot = path.parse(resolvedWorkspaceRoot).root;

  if (hostPlatform === 'win32' && workspaceDriveRoot) {
    return path.join(workspaceDriveRoot, directoryName);
  }

  const hostTempRoot = os.tmpdir();
  if (typeof hostTempRoot === 'string' && hostTempRoot.trim().length > 0) {
    return path.join(hostTempRoot, directoryName);
  }

  return path.join(resolvedWorkspaceRoot, 'bin', `.${directoryName}`);
}

export function resolveWorkspaceTargetDir({
  workspaceRoot,
  env = process.env,
  platform = process.platform,
  hostPlatform = process.platform,
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

  const managedWindowsTargetRoot = resolveConfiguredManagedWindowsTargetRoot(workspaceRoot, env);
  if (managedWindowsTargetRoot.length > 0) {
    return path.join(
      managedWindowsTargetRoot,
      'sdkwork-target',
      managedWindowsWorkspaceTargetLeaf(workspaceRoot),
    );
  }

  const defaultManagedWindowsTargetRoot = defaultManagedWindowsWorkspaceRoot(
    workspaceRoot,
    'sdkwork-target',
    hostPlatform,
  );
  if (defaultManagedWindowsTargetRoot.length > 0) {
    return path.join(
      defaultManagedWindowsTargetRoot,
      managedWindowsWorkspaceTargetLeaf(workspaceRoot),
    );
  }

  return path.join(workspaceRoot, 'bin', '.sdkwork-target-vs2022');
}

export function resolveWorkspaceTempDir({
  workspaceRoot,
  env = process.env,
  platform = process.platform,
  hostPlatform = process.platform,
} = {}) {
  if (typeof workspaceRoot !== 'string' || workspaceRoot.trim().length === 0) {
    throw new Error('workspaceRoot is required.');
  }

  if (platform !== 'win32') {
    return path.join(workspaceRoot, 'tmp');
  }

  const managedWindowsTempRoot = resolveConfiguredManagedWindowsTempRoot(workspaceRoot, env);
  if (managedWindowsTempRoot.length > 0) {
    return path.join(
      managedWindowsTempRoot,
      'sdkwork-temp',
      managedWindowsWorkspaceTargetLeaf(workspaceRoot),
    );
  }

  const defaultManagedWindowsTempRoot = defaultManagedWindowsWorkspaceRoot(
    workspaceRoot,
    'sdkwork-temp',
    hostPlatform,
  );
  if (defaultManagedWindowsTempRoot.length > 0) {
    return path.join(
      defaultManagedWindowsTempRoot,
      managedWindowsWorkspaceTargetLeaf(workspaceRoot),
    );
  }

  return path.join(workspaceRoot, 'bin', '.sdkwork-temp-vs2022');
}

export function withManagedWorkspaceTargetDir({
  workspaceRoot,
  env = process.env,
  platform = process.platform,
  hostPlatform = process.platform,
} = {}) {
  const nextEnv = { ...env };
  if (platform === 'win32' && String(nextEnv.CARGO_TARGET_DIR ?? '').trim().length === 0) {
    nextEnv.CARGO_TARGET_DIR = resolveWorkspaceTargetDir({
      workspaceRoot,
      env: nextEnv,
      platform,
      hostPlatform,
    });
  }

  return nextEnv;
}

export function withManagedWorkspaceTempDir({
  workspaceRoot,
  env = process.env,
  platform = process.platform,
  hostPlatform = process.platform,
} = {}) {
  const nextEnv = { ...env };
  if (platform === 'win32') {
    const tempDir = resolveWorkspaceTempDir({
      workspaceRoot,
      env: nextEnv,
      platform,
      hostPlatform,
    });
    mkdirSync(tempDir, { recursive: true });
    nextEnv.TEMP = tempDir;
    nextEnv.TMP = tempDir;
  }

  return nextEnv;
}
