import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

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

  assert.equal(plan[0].label, 'portal typecheck');
  assert.equal(plan[0].command, 'powershell.exe');
  assert.match(plan[0].args.join(' '), /typecheck/);
  assert.equal(plan[2].label, 'portal browser runtime smoke');
  assert.match(plan[2].args.join(' '), /check-portal-browser-runtime\.mjs/);
  assert.equal(plan[3].label, 'admin typecheck');
  assert.equal(plan[3].command, 'powershell.exe');
  assert.match(plan[3].args.join(' '), /typecheck/);
  assert.equal(plan[5].label, 'admin browser runtime smoke');
  assert.match(plan[5].args.join(' '), /check-admin-browser-runtime\.mjs/);
  assert.equal(plan[6].label, 'docs bootstrap safety');
  assert.match(plan[6].args.join(' '), /check-router-docs-safety\.mjs/);
  assert.equal(plan[7].label, 'workspace dependency audit');
  assert.match(plan[7].args.join(' '), /check-rust-dependency-audit\.mjs/);
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
