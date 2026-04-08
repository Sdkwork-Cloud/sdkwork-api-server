#!/usr/bin/env node

import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { withSupportedWindowsCmakeGenerator } from './run-tauri-cli.mjs';
import { withManagedWorkspaceTargetDir } from './workspace-target-dir.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export const VERIFICATION_GROUPS = [
  'interface-openapi',
  'gateway-service',
  'admin-service',
  'portal-service',
  'product-runtime',
  'workspace',
];

function resolveRustRunner(platform = process.platform, env = process.env) {
  if (platform === 'win32') {
    const rustupPath = path.join(env.USERPROFILE ?? '', '.cargo', 'bin', 'rustup.exe');
    if (existsSync(rustupPath)) {
      return {
        command: rustupPath,
        args: ['run', 'stable', 'cargo'],
        shell: false,
      };
    }

    return {
      command: 'rustup.exe',
      args: ['run', 'stable', 'cargo'],
      shell: true,
    };
  }

  return {
    command: 'rustup',
    args: ['run', 'stable', 'cargo'],
    shell: false,
  };
}

function verificationBaseEnv({
  workspaceRoot,
  platform = process.platform,
  env = process.env,
} = {}) {
  const baseEnv = withManagedWorkspaceTargetDir({
    workspaceRoot,
    env: withSupportedWindowsCmakeGenerator(env, platform),
    platform,
  });
  return {
    ...baseEnv,
    CARGO_TARGET_DIR: baseEnv.CARGO_TARGET_DIR ?? path.join(workspaceRoot, 'target'),
  };
}

function cargoStep({
  label,
  workspaceRoot,
  cargoArgs,
  platform = process.platform,
  env = process.env,
} = {}) {
  const rustRunner = resolveRustRunner(platform, env);
  return {
    label,
    command: rustRunner.command,
    args: [...rustRunner.args, ...cargoArgs],
    cwd: workspaceRoot,
    env: verificationBaseEnv({
      workspaceRoot,
      platform,
      env,
    }),
    shell: rustRunner.shell,
    windowsHide: platform === 'win32',
  };
}

export function createRustVerificationPlan({
  workspaceRoot = path.resolve(__dirname, '..'),
  group,
  platform = process.platform,
  env = process.env,
} = {}) {
  if (!VERIFICATION_GROUPS.includes(group)) {
    throw new Error(`unknown verification group: ${group}`);
  }

  switch (group) {
    case 'interface-openapi':
      return [
        cargoStep({
          label: 'gateway interface openapi route',
          workspaceRoot,
          cargoArgs: ['test', '-j', '1', '-p', 'sdkwork-api-interface-http', '--test', 'openapi_route'],
          platform,
          env,
        }),
        cargoStep({
          label: 'admin interface openapi route',
          workspaceRoot,
          cargoArgs: ['test', '-j', '1', '-p', 'sdkwork-api-interface-admin', '--test', 'openapi_route'],
          platform,
          env,
        }),
        cargoStep({
          label: 'portal interface openapi route',
          workspaceRoot,
          cargoArgs: ['test', '-j', '1', '-p', 'sdkwork-api-interface-portal', '--test', 'openapi_route'],
          platform,
          env,
        }),
      ];
    case 'gateway-service':
      return [
        cargoStep({
          label: 'gateway service cargo check',
          workspaceRoot,
          cargoArgs: ['check', '-j', '1', '-p', 'gateway-service'],
          platform,
          env,
        }),
      ];
    case 'admin-service':
      return [
        cargoStep({
          label: 'admin service cargo check',
          workspaceRoot,
          cargoArgs: ['check', '-j', '1', '-p', 'admin-api-service'],
          platform,
          env,
        }),
      ];
    case 'portal-service':
      return [
        cargoStep({
          label: 'portal service cargo check',
          workspaceRoot,
          cargoArgs: ['check', '-j', '1', '-p', 'portal-api-service'],
          platform,
          env,
        }),
      ];
    case 'product-runtime':
      return [
        cargoStep({
          label: 'product runtime cargo check',
          workspaceRoot,
          cargoArgs: ['check', '-j', '1', '-p', 'sdkwork-api-product-runtime'],
          platform,
          env,
        }),
        cargoStep({
          label: 'router product service cargo check',
          workspaceRoot,
          cargoArgs: ['check', '-j', '1', '-p', 'router-product-service'],
          platform,
          env,
        }),
      ];
    case 'workspace':
      return [
        cargoStep({
          label: 'workspace cargo check',
          workspaceRoot,
          cargoArgs: ['check', '--workspace', '-j', '1'],
          platform,
          env,
        }),
      ];
    default:
      throw new Error(`unhandled verification group: ${group}`);
  }
}

function parseArgs(argv = process.argv.slice(2)) {
  let group = '';
  let planFormat = '';

  for (let index = 0; index < argv.length; index += 1) {
    const value = argv[index];
    if (value === '--group') {
      group = argv[index + 1] ?? '';
      index += 1;
      continue;
    }
    if (value === '--plan-format') {
      planFormat = argv[index + 1] ?? '';
      index += 1;
      continue;
    }
    if (value === '--list-groups') {
      console.log(VERIFICATION_GROUPS.join('\n'));
      process.exit(0);
    }
    throw new Error(`unknown argument: ${value}`);
  }

  if (!group) {
    throw new Error('--group is required');
  }

  if (planFormat && planFormat !== 'json') {
    throw new Error(`unsupported plan format: ${planFormat}`);
  }

  return {
    group,
    planFormat,
  };
}

async function runStep(step) {
  await new Promise((resolve, reject) => {
    const child = spawn(step.command, step.args, {
      cwd: step.cwd,
      env: step.env,
      stdio: 'inherit',
      shell: step.shell ?? false,
      windowsHide: step.windowsHide ?? process.platform === 'win32',
    });

    child.on('error', reject);
    child.on('exit', (code, signal) => {
      if (signal) {
        reject(new Error(`${step.label} exited with signal ${signal}`));
        return;
      }
      if ((code ?? 1) !== 0) {
        reject(new Error(`${step.label} exited with code ${code}`));
        return;
      }
      resolve();
    });
  });
}

async function main() {
  const { group, planFormat } = parseArgs();
  const plan = createRustVerificationPlan({
    group,
  });

  if (planFormat === 'json') {
    console.log(JSON.stringify(plan, null, 2));
    return;
  }

  for (const step of plan) {
    console.error(`[check-rust-verification-matrix] ${step.label}`);
    // eslint-disable-next-line no-await-in-loop
    await runStep(step);
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch((error) => {
    console.error(`[check-rust-verification-matrix] ${error.message}`);
    process.exit(1);
  });
}
