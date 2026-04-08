import { existsSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  findReadableModuleResolution,
  importReadablePackageDefault,
  resolveReadablePackageRoot,
  resolveWorkspaceDonorRoots,
} from '../../scripts/dev/vite-runtime-lib.mjs';

const configDir = fileURLToPath(new URL('.', import.meta.url));
const installedUiRoot = path.join(configDir, 'node_modules', '@sdkwork', 'ui-pc-react');
const workspaceUiRoot = path.join(
  configDir,
  '..',
  '..',
  '..',
  'sdkwork-ui',
  'sdkwork-ui-pc-react',
);
const workspaceUiDistRoot = path.join(workspaceUiRoot, 'dist');
const defaultAdminProxyTarget = 'http://127.0.0.1:9981';
const donorRoots = resolveWorkspaceDonorRoots(configDir);
const normalizedConfigDir = path.resolve(configDir);

function resolveSdkworkUiDistPath(entryPath: string) {
  const installedCandidate = path.join(installedUiRoot, 'dist', entryPath);
  return existsSync(installedCandidate)
    ? installedCandidate
    : path.join(workspaceUiDistRoot, entryPath);
}

function resolveProxyTarget(envValue: string | undefined, fallbackTarget: string) {
  const trimmedValue = envValue?.trim();
  if (!trimmedValue) {
    return fallbackTarget;
  }

  return /^https?:\/\//i.test(trimmedValue)
    ? trimmedValue
    : `http://${trimmedValue}`;
}

const adminProxyTarget = resolveProxyTarget(
  process.env.SDKWORK_ADMIN_PROXY_TARGET ?? process.env.SDKWORK_ADMIN_BIND,
  defaultAdminProxyTarget,
);

const readableRuntimeDependencyAliases = [
  {
    find: /^react-router-dom$/,
    replacement: resolveReadablePackageRoot({
      appRoot: configDir,
      donorRoots,
      packageName: 'react-router-dom',
    }),
  },
  {
    find: /^react-router$/,
    replacement: resolveReadablePackageRoot({
      appRoot: configDir,
      donorRoots,
      packageName: 'react-router',
    }),
  },
];

type VitePluginFactory = () => unknown;

const sharedUiEntryAliases = [
  {
    find: /^motion\/react$/,
    replacement: path.join(configDir, 'src', 'vendor', 'motion-react.tsx'),
  },
  {
    find: /^@sdkwork\/ui-pc-react\/styles\.css$/,
    replacement: resolveSdkworkUiDistPath('sdkwork-ui.css'),
  },
  {
    find: /^@sdkwork\/ui-pc-react\/theme$/,
    replacement: resolveSdkworkUiDistPath('theme.js'),
  },
  {
    find: /^@sdkwork\/ui-pc-react\/components\/ui$/,
    replacement: resolveSdkworkUiDistPath('components-ui.js'),
  },
  {
    find: /^@sdkwork\/ui-pc-react\/components\/ui\/feedback$/,
    replacement: resolveSdkworkUiDistPath('ui-feedback.js'),
  },
  {
    find: /^@sdkwork\/ui-pc-react\/components\/patterns\/app-shell$/,
    replacement: resolveSdkworkUiDistPath('patterns-app-shell.js'),
  },
  {
    find: /^@sdkwork\/ui-pc-react\/components\/patterns\/desktop-shell$/,
    replacement: resolveSdkworkUiDistPath('patterns-desktop-shell.js'),
  },
  {
    find: /^@sdkwork\/ui-pc-react$/,
    replacement: resolveSdkworkUiDistPath('index.js'),
  },
];

function isBareModuleSpecifier(specifier: string) {
  return !specifier.startsWith('.')
    && !specifier.startsWith('/')
    && !specifier.startsWith('\0')
    && !specifier.startsWith('virtual:')
    && !specifier.startsWith('node:')
    && !specifier.startsWith('data:')
    && !/^[A-Za-z]:[\\/]/.test(specifier);
}

function shouldSkipReadableExternalFallback(specifier: string) {
  return specifier === 'motion/react'
    || specifier.startsWith('@sdkwork/')
    || specifier.startsWith('sdkwork-router-admin-');
}

function manualChunks(id: string) {
  if (!id.includes('node_modules')) {
    return undefined;
  }

  if (
    id.includes('\\react\\')
    || id.includes('/react/')
    || id.includes('\\react-dom\\')
    || id.includes('/react-dom/')
    || id.includes('\\react-router')
    || id.includes('/react-router')
    || id.includes('\\scheduler\\')
    || id.includes('/scheduler/')
    || id.includes('\\@remix-run\\router\\')
    || id.includes('/@remix-run/router/')
  ) {
    return 'react-vendor';
  }

  if (id.includes('\\@radix-ui\\') || id.includes('/@radix-ui/')) {
    return 'radix-vendor';
  }

  if (id.includes('\\lucide-react\\') || id.includes('/lucide-react/')) {
    return 'icon-vendor';
  }

  if (id.includes('\\motion\\') || id.includes('/motion/')) {
    return 'motion-vendor';
  }

  return undefined;
}

function readableExternalFallbackPlugin() {
  return {
    name: 'sdkwork-readable-external-fallback',
    enforce: 'pre' as const,
    resolveId(source: string) {
      if (!isBareModuleSpecifier(source) || shouldSkipReadableExternalFallback(source)) {
        return null;
      }

      try {
        const resolution = findReadableModuleResolution({
          appRoot: normalizedConfigDir,
          donorRoots,
          specifier: source,
        });

        return resolution.candidateRoot === normalizedConfigDir
          ? null
          : resolution.resolvedPath;
      } catch {
        return null;
      }
    },
  };
}

async function loadAdminVitePlugins() {
  const [tailwindcss, react] = await Promise.all([
    importReadablePackageDefault<VitePluginFactory>({
      appRoot: configDir,
      donorRoots,
      packageName: '@tailwindcss/vite',
      relativeEntry: ['dist', 'index.mjs'],
    }),
    importReadablePackageDefault<VitePluginFactory>({
      appRoot: configDir,
      donorRoots,
      packageName: '@vitejs/plugin-react',
      relativeEntry: ['dist', 'index.js'],
    }),
  ]);

  return {
    react,
    tailwindcss,
  };
}

async function defineAdminViteConfig() {
  const { react, tailwindcss } = await loadAdminVitePlugins();

  return {
    base: '/admin/',
    plugins: [readableExternalFallbackPlugin(), react(), tailwindcss()],
    build: {
      rollupOptions: {
        output: {
          manualChunks,
        },
      },
    },
    resolve: {
      dedupe: ['react', 'react-dom'],
      alias: [
        ...sharedUiEntryAliases,
        ...readableRuntimeDependencyAliases,
        {
          find: /^sdkwork-router-admin-apirouter$/,
          replacement: path.join(
            configDir,
            'packages',
            'sdkwork-router-admin-apirouter',
            'src',
            'index.ts',
          ),
        },
      ],
    },
    server: {
      host: '0.0.0.0',
      port: 5173,
      strictPort: true,
      proxy: {
        '/api/admin': {
          target: adminProxyTarget,
          changeOrigin: true,
          rewrite: (requestPath: string) => requestPath.replace(/^\/api\/admin/, '/admin'),
        },
      },
    },
    preview: {
      host: '0.0.0.0',
      port: 4173,
      strictPort: true,
    },
  };
}

export default await defineAdminViteConfig();
