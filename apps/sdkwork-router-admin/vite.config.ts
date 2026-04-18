import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  findReadableModuleResolution,
  importReadablePackageDefault,
  resolveReadablePackageRoot,
  resolveWorkspaceDonorRoots,
} from '../../scripts/dev/vite-runtime-lib.mjs';

const configDir = fileURLToPath(new URL('.', import.meta.url));
const sdkworkUiSourceRoot = path.resolve(
  configDir,
  '../../../sdkwork-ui/sdkwork-ui-pc-react/src',
);
const defaultAdminProxyTarget = 'http://127.0.0.1:9981';
const donorRoots = resolveWorkspaceDonorRoots(configDir);
const normalizedConfigDir = path.resolve(configDir);
const zustandPackageRoot = resolveReadablePackageRoot({
  appRoot: configDir,
  donorRoots,
  packageName: 'zustand',
});
const zustandEsmEntry = normalizeAliasPath(path.join(zustandPackageRoot, 'esm', 'index.mjs'));
const zustandEsmSubpathRoot = `${normalizeAliasPath(path.join(zustandPackageRoot, 'esm'))}/`;

function normalizeAliasPath(value: string) {
  return value.replaceAll('\\', '/');
}

function resolveSdkworkUiSourcePath(relativePath: string) {
  return path.resolve(sdkworkUiSourceRoot, relativePath);
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
  {
    find: /^zustand$/,
    replacement: zustandEsmEntry,
  },
  {
    find: /^zustand\//,
    replacement: zustandEsmSubpathRoot,
  },
];

type VitePluginFactory = () => unknown;

const sharedUiEntryAliases = [
  {
    find: /^motion\/react$/,
    replacement: path.join(configDir, 'src', 'vendor', 'motion-react.tsx'),
  },
  {
    find: /^use-sync-external-store\/shim$/,
    replacement: path.join(configDir, 'src', 'vendor', 'use-sync-external-store-shim.ts'),
  },
  {
    find: /^@sdkwork\/ui-pc-react\/styles\.css$/,
    replacement: resolveSdkworkUiSourcePath('styles/sdkwork-ui.css'),
  },
  {
    find: /^@sdkwork\/ui-pc-react\/theme$/,
    replacement: resolveSdkworkUiSourcePath('theme/index.ts'),
  },
  {
    find: /^@sdkwork\/ui-pc-react\/components\/ui$/,
    replacement: resolveSdkworkUiSourcePath('components/ui/index.ts'),
  },
  {
    find: /^@sdkwork\/ui-pc-react\/components\/ui\/feedback$/,
    replacement: resolveSdkworkUiSourcePath('components/ui/feedback/index.ts'),
  },
  {
    find: /^@sdkwork\/ui-pc-react\/components\/patterns\/app-shell$/,
    replacement: resolveSdkworkUiSourcePath('components/patterns/app-shell/index.ts'),
  },
  {
    find: /^@sdkwork\/ui-pc-react\/components\/patterns\/desktop-shell$/,
    replacement: resolveSdkworkUiSourcePath('components/patterns/desktop-shell/index.ts'),
  },
  {
    find: /^@sdkwork\/ui-pc-react$/,
    replacement: resolveSdkworkUiSourcePath('index.ts'),
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
      dedupe: ['react', 'react-dom', 'zustand'],
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
