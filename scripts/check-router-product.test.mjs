import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

import {
  resolveWorkspaceTargetDir,
  resolveWorkspaceTempDir,
} from './workspace-target-dir.mjs';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

function readJson(relativePath) {
  return JSON.parse(readFileSync(path.join(workspaceRoot, relativePath), 'utf8'));
}

function readText(relativePath) {
  return readFileSync(path.join(workspaceRoot, relativePath), 'utf8').replace(/\r\n/g, '\n');
}

test('check-router-product exposes Windows-safe pnpm and rust runner plans without ambient globals', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-router-product.mjs')).href,
  );

  const rustRunner = module.resolveRustRunner('win32', {
    USERPROFILE: process.env.USERPROFILE ?? '',
  });
  assert.equal(typeof rustRunner.command, 'string');
  assert.ok(Array.isArray(rustRunner.args));

  const plan = module.createProductCheckPlan({
    workspaceRoot,
    platform: 'win32',
    env: {},
  });
  const defaultWindowsTargetDir = resolveWorkspaceTargetDir({
    workspaceRoot,
    env: {},
    platform: 'win32',
    hostPlatform: 'win32',
  });
  const defaultWindowsTempDir = resolveWorkspaceTempDir({
    workspaceRoot,
    env: {},
    platform: 'win32',
    hostPlatform: 'win32',
  });

  const stepByLabel = new Map(plan.map((step) => [step.label, step]));

  assert.equal(stepByLabel.get('portal typecheck')?.command, process.execPath);
  assert.match(stepByLabel.get('portal typecheck')?.args.join(' ') ?? '', /run-tsc-cli\.mjs --noEmit/);
  assert.match(stepByLabel.get('portal browser runtime smoke')?.args.join(' ') ?? '', /check-portal-browser-runtime\.mjs/);
  assert.equal(stepByLabel.get('admin typecheck')?.command, process.execPath);
  assert.match(stepByLabel.get('admin typecheck')?.args.join(' ') ?? '', /run-tsc-cli\.mjs --noEmit/);
  assert.match(stepByLabel.get('admin browser runtime smoke')?.args.join(' ') ?? '', /check-admin-browser-runtime\.mjs/);
  assert.match(stepByLabel.get('docs bootstrap safety')?.args.join(' ') ?? '', /check-router-docs-safety\.mjs/);
  assert.equal(stepByLabel.get('docs site build')?.command, 'powershell.exe');
  assert.deepEqual(stepByLabel.get('docs site build')?.args.slice(0, 4), [
    '-NoProfile',
    '-ExecutionPolicy',
    'Bypass',
    '-Command',
  ]);
  assert.match(stepByLabel.get('docs site build')?.args[4] ?? '', /pnpm\.cjs/);
  assert.match(stepByLabel.get('docs site build')?.args[4] ?? '', /--dir/);
  assert.match(stepByLabel.get('docs site build')?.args[4] ?? '', /docs/);
  assert.match(stepByLabel.get('docs site build')?.args[4] ?? '', /build/);
  assert.equal(stepByLabel.get('docs site build')?.cwd, workspaceRoot);
  assert.equal(
    String(stepByLabel.get('docs site build')?.env.TEMP ?? '').replaceAll('\\', '/'),
    defaultWindowsTempDir.replaceAll('\\', '/'),
  );
  assert.equal(
    String(stepByLabel.get('docs site build')?.env.TMP ?? '').replaceAll('\\', '/'),
    defaultWindowsTempDir.replaceAll('\\', '/'),
  );
  assert.match(stepByLabel.get('workspace dependency audit')?.args.join(' ') ?? '', /check-rust-dependency-audit\.mjs/);
  assert.match(stepByLabel.get('portal desktop runtime payload')?.args.join(' ') ?? '', /prepare-router-portal-desktop-runtime\.mjs/);
  assert.doesNotMatch(stepByLabel.get('portal desktop runtime payload')?.args.join(' ') ?? '', /build-router-desktop-assets\.mjs/);
  assert.equal(
    String(stepByLabel.get('server cargo check')?.env.CARGO_TARGET_DIR ?? '').replaceAll('\\', '/'),
    defaultWindowsTargetDir.replaceAll('\\', '/'),
  );
  assert.equal(
    String(stepByLabel.get('server cargo check')?.env.TEMP ?? '').replaceAll('\\', '/'),
    defaultWindowsTempDir.replaceAll('\\', '/'),
  );
  assert.equal(
    String(stepByLabel.get('server cargo check')?.env.TMP ?? '').replaceAll('\\', '/'),
    defaultWindowsTempDir.replaceAll('\\', '/'),
  );
  assert.match(stepByLabel.get('server deployment plan')?.args.join(' ') ?? '', /--bind 127\.0\.0\.1:3001/);

  const linuxPlan = module.createProductCheckPlan({
    workspaceRoot,
    platform: 'linux',
    env: {
      CMAKE_GENERATOR: 'Visual Studio 17 2022',
      HOST_CMAKE_GENERATOR: 'Visual Studio 17 2022',
    },
  });
  assert.equal(Object.hasOwn(linuxPlan[0].env, 'CMAKE_GENERATOR'), false);
  assert.equal(Object.hasOwn(linuxPlan[0].env, 'HOST_CMAKE_GENERATOR'), false);
  assert.equal(
    linuxPlan.some((step) => step.args.join(' ').includes('build-router-desktop-assets.mjs')),
    false,
  );
});

test('workspace TypeScript app configs keep ignoreDeprecations compatible with the pinned compiler major', () => {
  for (const appRoot of [
    'apps/sdkwork-router-portal',
    'apps/sdkwork-router-admin',
    'console',
  ]) {
    const packageJson = readJson(path.join(appRoot, 'package.json'));
    const tsconfig = readJson(path.join(appRoot, 'tsconfig.json'));
    const typescriptVersion = packageJson.devDependencies?.typescript;

    assert.equal(
      typeof typescriptVersion,
      'string',
      `${appRoot} must pin a TypeScript compiler version`,
    );
    assert.match(
      typescriptVersion,
      /^5\./,
      `${appRoot} is expected to stay on the TypeScript 5.x toolchain for the current workspace`,
    );
    assert.equal(
      tsconfig.compilerOptions?.ignoreDeprecations,
      '5.0',
      `${appRoot} must keep ignoreDeprecations aligned with the pinned TypeScript 5.x compiler`,
    );
  }
});

test('product frontend package manifests explicitly allow esbuild build scripts under pnpm strict installs', () => {
  for (const appRoot of [
    'apps/sdkwork-router-portal',
    'apps/sdkwork-router-admin',
    'docs',
  ]) {
    const packageJson = readJson(path.join(appRoot, 'package.json'));
    const onlyBuiltDependencies = packageJson.pnpm?.onlyBuiltDependencies;

    assert.ok(
      Array.isArray(onlyBuiltDependencies),
      `${appRoot} must declare pnpm.onlyBuiltDependencies for deterministic CI installs`,
    );
    assert.ok(
      onlyBuiltDependencies.includes('esbuild'),
      `${appRoot} must explicitly allow esbuild build scripts for strict CI installs`,
    );
  }
});

test('workspace cargo config does not pin Windows-only CMake generators globally', () => {
  const cargoConfig = readText('.cargo/config.toml');

  assert.doesNotMatch(cargoConfig, /^\s*CMAKE_GENERATOR\s*=/m);
  assert.doesNotMatch(cargoConfig, /^\s*HOST_CMAKE_GENERATOR\s*=/m);
});

test('vendored pingora-core patch avoids the known Windows cargo-check dead-code warning patterns', () => {
  const connectorsL4 = readText('vendor/pingora-core-0.8.0/src/connectors/l4.rs');
  const listenersL4 = readText('vendor/pingora-core-0.8.0/src/listeners/l4.rs');
  const listenersMod = readText('vendor/pingora-core-0.8.0/src/listeners/mod.rs');
  const socket = readText('vendor/pingora-core-0.8.0/src/protocols/l4/socket.rs');
  const stream = readText('vendor/pingora-core-0.8.0/src/protocols/l4/stream.rs');
  const ext = readText('vendor/pingora-core-0.8.0/src/protocols/l4/ext.rs');
  const protocolsMod = readText('vendor/pingora-core-0.8.0/src/protocols/mod.rs');
  const bootstrapServices = readText('vendor/pingora-core-0.8.0/src/server/bootstrap_services.rs');
  const servicesListening = readText('vendor/pingora-core-0.8.0/src/services/listening.rs');
  const upstreamPeer = readText('vendor/pingora-core-0.8.0/src/upstreams/peer.rs');

  assert.match(connectorsL4, /#\[cfg\(unix\)\]\s*use crate::protocols::raw_connect;/);
  assert.match(connectorsL4, /async fn proxy_connect<P: Peer>\(_peer: &P\) -> Result<Stream>/);

  assert.match(
    listenersL4,
    /fn tcp_sock_opts\(&self\) -> Option<&TcpSocketOptions> \{[\s\S]*Self::Tcp\(_, op\) => op\.into\(\),[\s\S]*#\[cfg\(unix\)\]\s*Self::Uds\(_, _\) => None,/,
  );
  assert.doesNotMatch(listenersL4, /\n\s*_ => None,/);

  assert.match(listenersMod, /use std::\{any::Any, sync::Arc\};/);
  assert.match(listenersMod, /#\[cfg\(unix\)\]\s*use std::fs::Permissions;/);

  assert.doesNotMatch(socket, /if let SocketAddr::Inet\(addr\) = self/);
  assert.doesNotMatch(stream, /_ => \(\),/);
  assert.doesNotMatch(protocolsMod, /use std::\{net::SocketAddr as InetSocketAddr, path::Path\};/);

  assert.match(bootstrapServices, /#\[cfg\(unix\)\]\s*use tokio::sync::Mutex as TokioMutex;/);
  assert.match(
    bootstrapServices,
    /pub fn new\(\s*options: &Option<Opt>,\s*_conf: &ServerConf,\s*execution_phase_watch: &broadcast::Sender<ExecutionPhase>,/,
  );
  assert.match(bootstrapServices, /#\[cfg\(unix\)\]\s*upgrade: bool,/);
  assert.match(bootstrapServices, /#\[cfg\(unix\)\]\s*upgrade_sock: String,/);

  assert.doesNotMatch(ext, /#\[cfg\(windows\)\]\s*fn ip_local_port_range\(_fd: RawSocket, _low: u16, _high: u16\) -> io::Result<\(\)> \{/);
  assert.match(servicesListening, /#\[cfg\(unix\)\]\s*use std::fs::Permissions;/);
  const clientCertKeyStart = upstreamPeer.lastIndexOf(
    '\n\n    fn get_client_cert_key(&self) -> Option<&Arc<CertKey>> {',
  );
  assert.notEqual(
    clientCertKeyStart,
    -1,
    'upstreams/peer.rs must keep get_client_cert_key in the expected Peer implementation block',
  );
  const windowsMatchesSockStart = upstreamPeer.lastIndexOf(
    '#[cfg(windows)]\n    fn matches_sock<V: AsRawSocket>(&self, sock: V) -> bool {',
    clientCertKeyStart,
  );
  assert.notEqual(
    windowsMatchesSockStart,
    -1,
    'upstreams/peer.rs must define the Windows matches_sock helper before get_client_cert_key',
  );
  const windowsMatchesSock = upstreamPeer.slice(windowsMatchesSockStart, clientCertKeyStart);
  assert.match(
    windowsMatchesSock,
    /if self\.get_proxy\(\)\.is_some\(\) \{/,
  );
  assert.doesNotMatch(
    windowsMatchesSock,
    /if let Some\(proxy\) = self\.get_proxy\(\)/,
  );
});
