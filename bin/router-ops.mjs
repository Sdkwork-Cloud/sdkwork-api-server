#!/usr/bin/env node

import process from 'node:process';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

import {
  applyInstallPlan,
  assertInstallInputsExist,
  createInstallPlan,
  createReleaseBuildPlan,
  createValidateConfigPlan,
  defaultInstallRoot,
  executeValidateConfigPlan,
  executeReleaseBuildPlan,
  toPortablePath,
} from './lib/router-runtime-tooling.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const BUILD_ONLY_FLAGS = new Set(['--install', '--skip-docs', '--skip-console']);
const RUNTIME_LAYOUT_FLAGS = new Set(['--home', '--mode']);
const INSTALL_ONLY_FLAGS = new Set(['--force']);
const INSTALL_MODES = new Set(['portable', 'system']);

class UserInputError extends Error {}

function printUsage() {
  console.error(
    [
      'Usage:',
      '  node bin/router-ops.mjs build [--install] [--skip-docs] [--skip-console] [--dry-run]',
      '  node bin/router-ops.mjs install [--mode <portable|system>] [--home <dir>] [--force] [--dry-run]',
      '  node bin/router-ops.mjs validate-config [--mode <portable|system>] [--home <dir>] [--dry-run]',
    ].join('\n'),
  );
}

function assertOptionSupported(command, token) {
  if (BUILD_ONLY_FLAGS.has(token) && command !== 'build') {
    throw new UserInputError(`${token} is only supported for the build command`);
  }

  if (RUNTIME_LAYOUT_FLAGS.has(token) && command !== 'install' && command !== 'validate-config') {
    throw new UserInputError(`${token} is only supported for install or validate-config`);
  }

  if (INSTALL_ONLY_FLAGS.has(token) && command !== 'install') {
    throw new UserInputError(`${token} is only supported for the install command`);
  }
}

function readOptionValue(token, next) {
  if (!next || next.startsWith('--')) {
    throw new UserInputError(`${token} requires a value`);
  }

  return next;
}

export function parseArgs(argv) {
  const [command, ...rest] = argv;
  const options = {
    command,
    mode: 'portable',
    installDependencies: false,
    includeDocs: true,
    includeConsole: true,
    force: false,
    dryRun: false,
    installRoot: null,
  };

  for (let index = 0; index < rest.length; index += 1) {
    const token = rest[index];
    const next = rest[index + 1];

    if (!token.startsWith('--')) {
      throw new UserInputError(`unknown argument: ${token}`);
    }

    assertOptionSupported(command, token);

    switch (token) {
      case '--install':
        options.installDependencies = true;
        break;
      case '--skip-docs':
        options.includeDocs = false;
        break;
      case '--skip-console':
        options.includeConsole = false;
        break;
      case '--force':
        options.force = true;
        break;
      case '--dry-run':
        options.dryRun = true;
        break;
      case '--mode': {
        const mode = String(readOptionValue(token, next)).trim().toLowerCase();
        if (!INSTALL_MODES.has(mode)) {
          throw new UserInputError(`unsupported install mode: ${mode}`);
        }
        options.mode = mode;
        index += 1;
        break;
      }
      case '--home':
        options.installRoot = path.resolve(readOptionValue(token, next));
        index += 1;
        break;
      default:
        throw new UserInputError(`unknown option: ${token}`);
    }
  }

  if ((options.command === 'install' || options.command === 'validate-config') && !options.installRoot) {
    options.installRoot = defaultInstallRoot(repoRoot, {
      mode: options.mode,
      platform: process.platform,
      env: process.env,
    });
  }

  return options;
}

function printBuildPlan(plan) {
  console.log(`# target=${plan.target.targetTriple}`);
  for (const step of plan.steps) {
    console.log(`${step.label}: ${step.command} ${step.args.join(' ')}`);
  }
}

function printInstallPlan(plan) {
  console.log(`# install-root=${toPortablePath(plan.directories[0])}`);
  for (const directoryPath of plan.directories) {
    console.log(`mkdir ${toPortablePath(directoryPath)}`);
  }
  for (const file of plan.files) {
    if (file.type === 'text') {
      console.log(`write ${toPortablePath(file.targetPath)}`);
      continue;
    }
    console.log(`copy ${toPortablePath(file.sourcePath)} -> ${toPortablePath(file.targetPath)}`);
  }
}

function printValidateConfigPlan(plan) {
  console.log(`# validate-config mode=${plan.mode} source=${plan.source}`);
  console.log(`# install-root=${toPortablePath(plan.installRoot)}`);
  console.log(`${plan.label}: ${plan.command} ${plan.args.join(' ')}`);
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  if (!options.command) {
    printUsage();
    process.exit(1);
  }

  if (options.command === 'build') {
    const plan = createReleaseBuildPlan({
      repoRoot,
      installDependencies: options.installDependencies,
      includeDocs: options.includeDocs,
      includeConsole: options.includeConsole,
    });

    if (options.dryRun) {
      printBuildPlan(plan);
      return;
    }

    await executeReleaseBuildPlan(plan);
    return;
  }

  if (options.command === 'install') {
    const plan = createInstallPlan({
      repoRoot,
      mode: options.mode,
      installRoot: options.installRoot,
    });

    if (options.dryRun) {
      printInstallPlan(plan);
      return;
    }

    assertInstallInputsExist(plan);

    applyInstallPlan(plan, {
      force: options.force,
    });
    console.log(`installed runtime to ${toPortablePath(options.installRoot)}`);
    return;
  }

  if (options.command === 'validate-config') {
    const plan = createValidateConfigPlan({
      repoRoot,
      mode: options.mode,
      installRoot: options.installRoot,
    });

    if (options.dryRun) {
      printValidateConfigPlan(plan);
      return;
    }

    await executeValidateConfigPlan(plan);
    console.log(`validated runtime config for ${toPortablePath(plan.installRoot)}`);
    return;
  }

  printUsage();
  process.exit(1);
}

function handleFatalError(error) {
  if (error instanceof UserInputError) {
    console.error(error.message);
    printUsage();
    process.exit(1);
  }

  console.error(error instanceof Error ? error.stack ?? error.message : String(error));
  process.exit(1);
}

function isDirectExecution() {
  if (!process.argv[1]) {
    return false;
  }

  return pathToFileURL(path.resolve(process.argv[1])).href === import.meta.url;
}

if (isDirectExecution()) {
  main().catch(handleFatalError);
}
