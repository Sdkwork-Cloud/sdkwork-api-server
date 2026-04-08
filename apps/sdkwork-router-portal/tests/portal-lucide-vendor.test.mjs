import assert from 'node:assert/strict';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function resolvePackageRoot(packageName) {
  const directRoot = path.join(appRoot, 'node_modules', packageName);
  if (existsSync(path.join(directRoot, 'package.json'))) {
    return directRoot;
  }

  const pnpmRoot = path.join(appRoot, 'node_modules', '.pnpm');
  if (!existsSync(pnpmRoot)) {
    throw new Error(`unable to find pnpm store for ${packageName}`);
  }

  const candidates = readdirSync(pnpmRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory() && entry.name.startsWith(`${packageName}@`))
    .map((entry) => entry.name)
    .sort()
    .reverse();

  for (const candidate of candidates) {
    const packageRoot = path.join(pnpmRoot, candidate, 'node_modules', packageName);
    if (existsSync(path.join(packageRoot, 'package.json'))) {
      return packageRoot;
    }
  }

  throw new Error(`unable to resolve ${packageName} package root`);
}

function collectLucideImports(source) {
  const imports = new Set();
  const regex = /import\s*\{([^}]+)\}\s*from\s*['"]lucide-react['"]/g;

  for (const match of source.matchAll(regex)) {
    const specifiers = match[1]
      .split(',')
      .map((value) => value.trim())
      .filter(Boolean);

    for (const specifier of specifiers) {
      if (specifier.startsWith('type ')) {
        continue;
      }

      const [imported] = specifier.split(/\s+as\s+/i);
      if (imported && imported !== 'LucideIcon') {
        imports.add(imported.trim());
      }
    }
  }

  return imports;
}

function collectVendorIconFiles(source) {
  return [...source.matchAll(/import\s+\w+\s+from\s+['"]lucide-react\/dist\/esm\/icons\/([^'"]+)['"]/g)]
    .map((match) => match[1]);
}

test('lucide vendor exports every runtime icon consumed by portal and linked sdkwork-ui sources', () => {
  const vendorSource = read('src/vendor/lucide-react.ts');
  const runtimeSources = [
    'packages/sdkwork-router-portal-commons/src/framework/form.tsx',
    'packages/sdkwork-router-portal-core/src/components/PortalNavigationRail.tsx',
    'packages/sdkwork-router-portal-core/src/components/PortalSettingsCenter.tsx',
    'packages/sdkwork-router-portal-core/src/components/WindowControls.tsx',
    'packages/sdkwork-router-portal-core/src/components/settings/PortalSettingsPrimitives.tsx',
    'packages/sdkwork-router-portal-auth/src/pages/AuthPage.tsx',
    'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyDrawers.tsx',
    'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyCreateForm.tsx',
    'packages/sdkwork-router-portal-api-keys/src/components/ApiKeyManagedNoticeCard.tsx',
    'packages/sdkwork-router-portal-dashboard/src/pages/index.tsx',
    'packages/sdkwork-router-portal-usage/src/pages/index.tsx',
    'packages/sdkwork-router-portal-credits/src/pages/index.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/dropdown-menu.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/dialog.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/data-display/tree.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/actions/command.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/actions/split-button.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/actions/bulk-action-bar.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/checkbox.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/actions/action-menu-button.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/form/settings-field.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/button.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/breadcrumb.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/data-entry/date-input.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/data-entry/combobox.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/data-entry/date-range-picker.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/data-entry/number-input.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/data-entry/tag-input.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/data-entry/upload/upload-item.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/data-entry/upload/upload-dropzone.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/feedback/activity-feed.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/data-display/data-table/header-cell.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/status-badge.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/feedback/empty-search.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/pagination.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/feedback/inline-alert.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/feedback/states.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/select.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/feedback/notification-center.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/radio-group.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/navigation/menubar.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/overlays/modal.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/patterns/settings/SettingsCenter.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/navigation/workspace-tabs.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/layout/sidebar-section.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/overlays/drawer.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/ui/overlays/context-menu.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/patterns/picker/TwoPaneSelectorPopover.tsx',
    '../../../sdkwork-ui/sdkwork-ui-pc-react/src/components/patterns/desktop-shell/DesktopWindowControls.tsx',
  ];

  const importedIcons = new Set();

  for (const relativePath of runtimeSources) {
    const source = read(relativePath);
    for (const icon of collectLucideImports(source)) {
      importedIcons.add(icon);
    }
  }

  for (const icon of importedIcons) {
    assert.match(
      vendorSource,
      new RegExp(`\\b${icon}\\b`),
      `missing lucide vendor export for ${icon}`,
    );
  }
});

test('lucide vendor only imports icon files that exist in the pinned lucide-react package', () => {
  const vendorSource = read('src/vendor/lucide-react.ts');
  const lucideRoot = resolvePackageRoot('lucide-react');
  const iconsRoot = path.join(lucideRoot, 'dist', 'esm', 'icons');

  for (const iconFile of collectVendorIconFiles(vendorSource)) {
    const iconPath = path.join(iconsRoot, iconFile);
    assert.equal(
      existsSync(iconPath),
      true,
      `missing lucide icon file ${iconFile} under ${iconsRoot}`,
    );
  }
});
