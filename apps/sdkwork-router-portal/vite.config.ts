import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { createRequire } from 'node:module';
import { existsSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vite';

const configDir = fileURLToPath(new URL('.', import.meta.url));
const appRequire = createRequire(import.meta.url);
const sdkworkUiSourceRoot = path.resolve(
  configDir,
  '../../../sdkwork-ui/sdkwork-ui-pc-react/src',
);
const defaultAdminProxyTarget = 'http://127.0.0.1:9981';
const defaultPortalProxyTarget = 'http://127.0.0.1:9982';

function resolveAppDependency(specifier: string) {
  return appRequire.resolve(specifier);
}

function resolveSdkworkUiSourcePath(relativePath: string) {
  return path.resolve(sdkworkUiSourceRoot, relativePath);
}

function normalizeAliasPath(value: string) {
  return value.replaceAll('\\', '/');
}

function resolvePnpmPackageRoot(packageName: string) {
  const directRoot = path.resolve(configDir, 'node_modules', packageName);
  if (existsSync(path.join(directRoot, 'package.json'))) {
    return directRoot;
  }

  const pnpmRoot = path.resolve(configDir, 'node_modules', '.pnpm');
  const candidateNames = existsSync(pnpmRoot)
    ? readdirSync(pnpmRoot, { withFileTypes: true })
        .filter((entry) => entry.isDirectory() && entry.name.startsWith(`${packageName}@`))
        .map((entry) => entry.name)
        .sort()
        .reverse()
    : [];

  for (const candidateName of candidateNames) {
    const candidateRoot = path.join(pnpmRoot, candidateName, 'node_modules', packageName);
    if (existsSync(path.join(candidateRoot, 'package.json'))) {
      return candidateRoot;
    }
  }

  throw new Error(`unable to resolve ${packageName} from ${configDir}`);
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
const portalProxyTarget = resolveProxyTarget(
  process.env.SDKWORK_PORTAL_PROXY_TARGET ?? process.env.SDKWORK_PORTAL_BIND,
  defaultPortalProxyTarget,
);
const lucideReactRoot = resolvePnpmPackageRoot('lucide-react');
const lucideReactIconsRoot = `${normalizeAliasPath(path.join(lucideReactRoot, 'dist', 'esm', 'icons'))}/`;

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

  if (
    id.includes('\\class-variance-authority\\')
    || id.includes('/class-variance-authority/')
    || id.includes('\\tailwind-merge\\')
    || id.includes('/tailwind-merge/')
    || id.includes('\\clsx\\')
    || id.includes('/clsx/')
  ) {
    return 'style-vendor';
  }

  if (id.includes('\\@radix-ui\\') || id.includes('/@radix-ui/')) {
    return 'radix-vendor';
  }

  if (id.includes('\\recharts\\') || id.includes('/recharts/')) {
    return 'charts-vendor';
  }

  return undefined;
}

export default defineConfig({
  base: '/portal/',
  resolve: {
    // Keep the linked sdkwork-ui workspace sources on the same runtime singletons as the portal host.
    dedupe: [
      'react',
      'react-dom',
      'react-router-dom',
      'lucide-react',
      'motion',
      'framer-motion',
      'zustand',
    ],
    alias: [
      {
        find: /^react$/,
        replacement: resolveAppDependency('react'),
      },
      {
        find: /^react-dom$/,
        replacement: resolveAppDependency('react-dom'),
      },
      {
        find: /^react-dom\/client$/,
        replacement: resolveAppDependency('react-dom/client'),
      },
      {
        find: /^react\/jsx-runtime$/,
        replacement: resolveAppDependency('react/jsx-runtime'),
      },
      {
        find: /^react\/jsx-dev-runtime$/,
        replacement: resolveAppDependency('react/jsx-dev-runtime'),
      },
      {
        find: /^lucide-react$/,
        replacement: path.resolve(configDir, './src/vendor/lucide-react.ts'),
      },
      {
        find: /^lucide-react\/dist\/esm\/icons\//,
        replacement: lucideReactIconsRoot,
      },
      {
        find: /^@sdkwork\/ui-pc-react\/theme$/,
        replacement: resolveSdkworkUiSourcePath('theme/index.ts'),
      },
      {
        find: /^@sdkwork\/ui-pc-react\/components\/ui\/actions$/,
        replacement: resolveSdkworkUiSourcePath('components/ui/actions/index.ts'),
      },
      {
        find: /^@sdkwork\/ui-pc-react\/components\/ui\/data-entry$/,
        replacement: resolveSdkworkUiSourcePath('components/ui/data-entry/index.ts'),
      },
      {
        find: /^@sdkwork\/ui-pc-react\/components\/ui\/data-display$/,
        replacement: resolveSdkworkUiSourcePath('components/ui/data-display/index.ts'),
      },
      {
        find: /^@sdkwork\/ui-pc-react\/components\/ui\/form$/,
        replacement: resolveSdkworkUiSourcePath('components/ui/form/index.ts'),
      },
      {
        find: /^@sdkwork\/ui-pc-react\/components\/ui\/feedback$/,
        replacement: resolveSdkworkUiSourcePath('components/ui/feedback/index.ts'),
      },
      {
        find: /^@sdkwork\/ui-pc-react\/components\/ui\/layout$/,
        replacement: resolveSdkworkUiSourcePath('components/ui/layout/index.ts'),
      },
      {
        find: /^@sdkwork\/ui-pc-react\/components\/ui\/overlays$/,
        replacement: resolveSdkworkUiSourcePath('components/ui/overlays/index.ts'),
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
        find: /^@sdkwork\/ui-pc-react\/components\/patterns\/workbench$/,
        replacement: resolveSdkworkUiSourcePath('components/patterns/workbench/index.ts'),
      },
      {
        find: /^@sdkwork\/ui-pc-react\/components\/patterns\/workspace$/,
        replacement: resolveSdkworkUiSourcePath('components/patterns/workspace/index.ts'),
      },
      {
        find: /^motion\/react$/,
        replacement: path.resolve(
          configDir,
          './src/vendor/motion-react.tsx',
        ),
      },
      {
        find: /^sdkwork-router-portal-console$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-console/src/index.tsx'),
      },
      {
        find: /^sdkwork-router-portal-account$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-account/src/index.tsx'),
      },
      {
        find: /^sdkwork-router-portal-api-keys$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-api-keys/src/index.tsx'),
      },
      {
        find: /^sdkwork-router-portal-api-reference$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-api-reference/src/index.tsx'),
      },
      {
        find: /^sdkwork-router-portal-auth$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-auth/src/index.tsx'),
      },
      {
        find: /^sdkwork-router-portal-billing$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-billing/src/index.tsx'),
      },
      {
        find: /^sdkwork-router-portal-commons$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-commons/src/index.tsx'),
      },
      {
        find: /^sdkwork-router-portal-core$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-core/src/index.tsx'),
      },
      {
        find: /^sdkwork-router-portal-credits$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-credits/src/index.tsx'),
      },
      {
        find: /^sdkwork-router-portal-dashboard$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-dashboard/src/index.tsx'),
      },
      {
        find: /^sdkwork-router-portal-docs$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-docs/src/index.tsx'),
      },
      {
        find: /^sdkwork-router-portal-downloads$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-downloads/src/index.tsx'),
      },
      {
        find: /^sdkwork-router-portal-gateway$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-gateway/src/index.tsx'),
      },
      {
        find: /^sdkwork-router-portal-home$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-home/src/index.tsx'),
      },
      {
        find: /^sdkwork-router-portal-models$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-models/src/index.tsx'),
      },
      {
        find: /^sdkwork-router-portal-portal-api$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-portal-api/src/index.ts'),
      },
      {
        find: /^sdkwork-router-portal-recharge$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-recharge/src/index.tsx'),
      },
      {
        find: /^sdkwork-router-portal-routing$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-routing/src/index.tsx'),
      },
      {
        find: /^sdkwork-router-portal-settlements$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-settlements/src/index.tsx'),
      },
      {
        find: /^sdkwork-router-portal-types$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-types/src/index.ts'),
      },
      {
        find: /^sdkwork-router-portal-usage$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-usage/src/index.tsx'),
      },
      {
        find: /^sdkwork-router-portal-user$/,
        replacement: path.resolve(configDir, './packages/sdkwork-router-portal-user/src/index.tsx'),
      },
    ],
  },
  plugins: [react(), tailwindcss()],
  build: {
    rollupOptions: {
      output: {
        manualChunks,
      },
    },
  },
  server: {
    host: '0.0.0.0',
    port: 5174,
    strictPort: true,
    proxy: {
      '/api/admin': {
        target: adminProxyTarget,
        changeOrigin: true,
        rewrite: (sourcePath) => sourcePath.replace(/^\/api\/admin/, '/admin'),
      },
      '/api/portal': {
        target: portalProxyTarget,
        changeOrigin: true,
        rewrite: (sourcePath) => sourcePath.replace(/^\/api\/portal/, '/portal'),
      },
    },
  },
  preview: {
    host: '0.0.0.0',
    port: 4174,
    strictPort: true,
  },
});
