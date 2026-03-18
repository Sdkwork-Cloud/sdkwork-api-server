function requireValue(argv, index, flag) {
  const value = argv[index + 1];
  if (!value || value.startsWith('--')) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

export function parseWorkspaceArgs(argv) {
  const settings = {
    databaseUrl: null,
    gatewayBind: '127.0.0.1:8080',
    adminBind: '127.0.0.1:8081',
    portalBind: '127.0.0.1:8082',
    webBind: '0.0.0.0:3001',
    install: false,
    preview: false,
    tauri: false,
    dryRun: false,
    help: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];

    switch (arg) {
      case '--database-url':
        settings.databaseUrl = requireValue(argv, index, arg);
        index += 1;
        break;
      case '--gateway-bind':
        settings.gatewayBind = requireValue(argv, index, arg);
        index += 1;
        break;
      case '--admin-bind':
        settings.adminBind = requireValue(argv, index, arg);
        index += 1;
        break;
      case '--portal-bind':
        settings.portalBind = requireValue(argv, index, arg);
        index += 1;
        break;
      case '--web-bind':
        settings.webBind = requireValue(argv, index, arg);
        index += 1;
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
      case '--dry-run':
        settings.dryRun = true;
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

export function buildWorkspaceCommandPlan(settings) {
  const backendArgs = ['scripts/dev/start-stack.mjs'];
  if (settings.databaseUrl) {
    backendArgs.push('--database-url', settings.databaseUrl);
  }
  backendArgs.push(
    '--gateway-bind',
    settings.gatewayBind,
    '--admin-bind',
    settings.adminBind,
    '--portal-bind',
    settings.portalBind,
  );
  if (settings.dryRun) {
    backendArgs.push('--dry-run');
  }

  const adminArgs = ['scripts/dev/start-admin.mjs'];
  const portalArgs = ['scripts/dev/start-portal.mjs'];
  const webArgs = ['scripts/dev/start-web.mjs'];
  webArgs.push(
    '--bind',
    settings.webBind,
    '--admin-target',
    settings.adminBind,
    '--portal-target',
    settings.portalBind,
    '--gateway-target',
    settings.gatewayBind,
  );

  if (settings.install) {
    adminArgs.push('--install');
    portalArgs.push('--install');
    webArgs.push('--install');
  }

  if (settings.preview) {
    webArgs.push('--preview');
  }

  if (settings.tauri) {
    adminArgs.push('--tauri');
    webArgs.push('--tauri');
  }

  if (settings.dryRun) {
    adminArgs.push('--dry-run');
    portalArgs.push('--dry-run');
    webArgs.push('--dry-run');
  }

  return {
    nodeExecutable: process.execPath,
    backend: {
      name: 'backend',
      scriptPath: 'scripts/dev/start-stack.mjs',
      args: backendArgs,
    },
    admin: {
      name: settings.tauri ? 'admin-tauri' : 'admin-browser',
      scriptPath: 'scripts/dev/start-admin.mjs',
      args: adminArgs,
    },
    portal: {
      name: settings.preview ? 'portal-preview' : 'portal-browser',
      scriptPath: 'scripts/dev/start-portal.mjs',
      args: portalArgs,
    },
    web: {
      name: settings.preview ? 'web-preview' : settings.tauri ? 'web-tauri' : 'web-static',
      scriptPath: 'scripts/dev/start-web.mjs',
      args: webArgs,
    },
  };
}

export function workspaceHelpText() {
  return `Usage: node scripts/dev/start-workspace.mjs [options]

Starts the backend services plus the standalone admin and portal surfaces in one command.

Options:
  --database-url <url>   Optional shared SDKWORK_DATABASE_URL override for admin, gateway, and portal
  --gateway-bind <bind>  SDKWORK_GATEWAY_BIND override
  --admin-bind <bind>    SDKWORK_ADMIN_BIND override
  --portal-bind <bind>   SDKWORK_PORTAL_BIND override
  --web-bind <bind>      SDKWORK_WEB_BIND override for the Pingora public host
  --install              Run pnpm install before starting the frontend apps
  --preview              Build admin and portal, then serve them through the Pingora web host
  --tauri                Start the admin Tauri shell and the Pingora web host for external access
  --dry-run              Print the backend, admin, portal, and web-host commands without running them
  -h, --help             Show this help
`;
}
