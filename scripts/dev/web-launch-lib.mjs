import os from 'node:os';

function requireValue(argv, index, flag) {
  const value = argv[index + 1];
  if (!value || value.startsWith('--')) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

export function parseWebArgs(argv) {
  const settings = {
    adminTarget: '127.0.0.1:8081',
    bind: '0.0.0.0:3001',
    dryRun: false,
    gatewayTarget: '127.0.0.1:8080',
    help: false,
    install: false,
    portalTarget: '127.0.0.1:8082',
    preview: false,
    tauri: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    switch (arg) {
      case '--bind':
        settings.bind = requireValue(argv, index, arg);
        index += 1;
        break;
      case '--admin-target':
        settings.adminTarget = requireValue(argv, index, arg);
        index += 1;
        break;
      case '--portal-target':
        settings.portalTarget = requireValue(argv, index, arg);
        index += 1;
        break;
      case '--gateway-target':
        settings.gatewayTarget = requireValue(argv, index, arg);
        index += 1;
        break;
      case '--dry-run':
        settings.dryRun = true;
        break;
      case '--install':
        settings.install = true;
        break;
      case '--preview':
        settings.preview = true;
        break;
      case '--tauri':
        settings.tauri = true;
        break;
      case '--help':
      case '-h':
        settings.help = true;
        break;
      default:
        throw new Error(`unknown option: ${arg}`);
    }
  }

  return settings;
}

export function webHelpText() {
  return `Usage: node scripts/dev/start-web.mjs [options]

Builds the standalone admin and portal apps, then exposes them through the Pingora web host.

Options:
  --bind <bind> Use a specific SDKWORK_WEB_BIND value, default 0.0.0.0:3001
  --admin-target <host:port>   Upstream target for /api/admin/*, default 127.0.0.1:8081
  --portal-target <host:port>  Upstream target for /api/portal/*, default 127.0.0.1:8082
  --gateway-target <host:port> Upstream target for /api/v1/*, default 127.0.0.1:8080
  --install     Run pnpm install before starting
  --preview     Alias for static web-host mode
  --tauri       Build static assets for the admin Tauri host and external Pingora site
  --dry-run     Print the commands without running them
  -h, --help    Show this help
`;
}

export function webHostEnv(bind, targets = {}) {
  const {
    adminTarget = '127.0.0.1:8081',
    portalTarget = '127.0.0.1:8082',
    gatewayTarget = '127.0.0.1:8080',
  } = targets;

  return {
    ...process.env,
    SDKWORK_WEB_BIND: bind,
    SDKWORK_ADMIN_SITE_DIR: 'apps/sdkwork-router-admin/dist',
    SDKWORK_PORTAL_SITE_DIR: 'apps/sdkwork-router-portal/dist',
    SDKWORK_ADMIN_PROXY_TARGET: adminTarget,
    SDKWORK_PORTAL_PROXY_TARGET: portalTarget,
    SDKWORK_GATEWAY_PROXY_TARGET: gatewayTarget,
  };
}

export function publicEntryUrls(bind) {
  const [host, port = '3001'] = bind.split(':');
  const urls = [];

  if (host === '0.0.0.0') {
    urls.push(`http://127.0.0.1:${port}`);
    try {
      for (const interfaces of Object.values(os.networkInterfaces())) {
        for (const network of interfaces ?? []) {
          if (network.family === 'IPv4' && !network.internal) {
            urls.push(`http://${network.address}:${port}`);
          }
        }
      }
    } catch {
      // Some runtimes cannot enumerate interfaces, but the public bind is still valid.
    }
  } else {
    urls.push(`http://${host}:${port}`);
  }

  return [...new Set(urls)];
}

export function webAccessLines(bind) {
  const lines = [`[start-web] SDKWORK_WEB_BIND=${bind}`];
  for (const baseUrl of publicEntryUrls(bind)) {
    lines.push(`[start-web] Pingora admin: ${baseUrl}/admin/`);
    lines.push(`[start-web] Pingora portal: ${baseUrl}/portal/`);
  }
  return lines;
}
