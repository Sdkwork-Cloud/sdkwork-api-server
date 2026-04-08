import assert from 'node:assert/strict';
import { closeSync, existsSync, openSync, readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const appRoot = path.resolve(import.meta.dirname, '..');
const packageRoot = path.join(appRoot, 'packages');
const ignoredDirectories = new Set(['build', 'dist', 'node_modules']);
const packageRoots = [
  appRoot,
  path.resolve(appRoot, '..', 'sdkwork-router-portal'),
];

function isReadable(entryPath) {
  try {
    const descriptor = openSync(entryPath, 'r');
    closeSync(descriptor);
    return true;
  } catch {
    return false;
  }
}

function resolvePnpmPackageEntry(packageName, entryPath) {
  const packagePrefix = packageName.startsWith('@')
    ? `${packageName.slice(1).replace('/', '+')}@`
    : `${packageName}@`;

  for (const root of packageRoots) {
    const linkedEntry = path.join(root, 'node_modules', packageName, entryPath);
    if (existsSync(linkedEntry) && isReadable(linkedEntry)) {
      return linkedEntry;
    }

    const pnpmRoot = path.join(root, 'node_modules', '.pnpm');
    if (!existsSync(pnpmRoot)) {
      continue;
    }

    for (const entry of readdirSync(pnpmRoot, { withFileTypes: true })) {
      if (!entry.isDirectory() || !entry.name.startsWith(packagePrefix)) {
        continue;
      }

      const resolvedEntry = path.join(
        pnpmRoot,
        entry.name,
        'node_modules',
        packageName,
        entryPath,
      );
      if (existsSync(resolvedEntry) && isReadable(resolvedEntry)) {
        return resolvedEntry;
      }
    }
  }

  throw new Error(`Unable to resolve ${packageName}/${entryPath} from pnpm workspace layout`);
}

const tsModule = await import(
  pathToFileURL(resolvePnpmPackageEntry('typescript', path.join('lib', 'typescript.js'))).href
);
const ts = tsModule.default ?? tsModule;
const { default: jiti } = await import(
  pathToFileURL(resolvePnpmPackageEntry('jiti', path.join('lib', 'jiti.mjs'))).href
);
const loadTsModule = jiti(import.meta.url, { moduleCache: false });

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function listAdminSourceFiles(directory = packageRoot, files = []) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (ignoredDirectories.has(entry.name)) {
      continue;
    }

    const fullPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      listAdminSourceFiles(fullPath, files);
      continue;
    }

    if (
      fullPath.includes(`${path.sep}src${path.sep}`) &&
      (fullPath.endsWith('.ts') || fullPath.endsWith('.tsx'))
    ) {
      files.push(path.relative(appRoot, fullPath).replaceAll('\\', '/'));
    }
  }

  return files;
}

function hasVisibleTextCandidate(text) {
  const trimmed = text.trim();
  if (!trimmed || trimmed.length < 2) {
    return false;
  }

  if (/^(use[A-Z]|on[A-Z]|data-|aria-)/.test(trimmed)) {
    return false;
  }

  if (/^https?:/i.test(trimmed)) {
    return false;
  }

  if (/^[./@][A-Za-z0-9._/-]+$/.test(trimmed)) {
    return false;
  }

  if (/^[a-z]+(?:[A-Z][A-Za-z0-9]*)+$/.test(trimmed)) {
    return false;
  }

  return /[A-Za-z\u4e00-\u9fff]/.test(trimmed);
}

function collectVisibleTextCandidates(relativePath) {
  const source = read(relativePath);
  const sourceFile = ts.createSourceFile(
    relativePath,
    source,
    ts.ScriptTarget.Latest,
    true,
    relativePath.endsWith('.tsx') ? ts.ScriptKind.TSX : ts.ScriptKind.TS,
  );
  const attrNames = new Set([
    'aria-label',
    'cancelText',
    'confirmText',
    'description',
    'emptyMessage',
    'helperText',
    'label',
    'placeholder',
    'subtitle',
    'title',
    'tooltip',
  ]);
  const candidates = [];

  function visit(node) {
    if (ts.isJsxText(node)) {
      const text = node.getText(sourceFile).trim();
      if (hasVisibleTextCandidate(text)) {
        candidates.push(text);
      }
    }

    if (ts.isJsxAttribute(node) && node.initializer) {
      const name = node.name.getText(sourceFile);
      if (attrNames.has(name)) {
        if (ts.isStringLiteral(node.initializer) || ts.isNoSubstitutionTemplateLiteral(node.initializer)) {
          const text = node.initializer.text;
          if (hasVisibleTextCandidate(text)) {
            candidates.push(text);
          }
        }

        if (ts.isJsxExpression(node.initializer)) {
          const { expression } = node.initializer;
          if (
            expression &&
            (ts.isStringLiteral(expression) || ts.isNoSubstitutionTemplateLiteral(expression)) &&
            hasVisibleTextCandidate(expression.text)
          ) {
            candidates.push(expression.text);
          }
        }
      }
    }

    ts.forEachChild(node, visit);
  }

  visit(sourceFile);

  return candidates;
}

function collectUsedTranslationKeys() {
  const keys = new Set();
  const helperNames = new Set(['t', 'translateAdminText']);

  for (const relativePath of listAdminSourceFiles()) {
    const source = read(relativePath);
    const sourceFile = ts.createSourceFile(
      relativePath,
      source,
      ts.ScriptTarget.Latest,
      true,
      relativePath.endsWith('.tsx') ? ts.ScriptKind.TSX : ts.ScriptKind.TS,
    );

    function visit(node) {
      if (ts.isCallExpression(node)) {
        const callee = ts.isIdentifier(node.expression)
          ? node.expression.text
          : node.expression.getText(sourceFile);

        if (helperNames.has(callee) && node.arguments.length > 0) {
          const [firstArgument] = node.arguments;
          if (
            ts.isStringLiteral(firstArgument) ||
            ts.isNoSubstitutionTemplateLiteral(firstArgument)
          ) {
            keys.add(firstArgument.text);
          }
        }
      }

      ts.forEachChild(node, visit);
    }

    visit(sourceFile);
  }

  return [...keys].sort((left, right) => left.localeCompare(right));
}

function getPropertyNameText(name) {
  if (
    ts.isIdentifier(name) ||
    ts.isStringLiteral(name) ||
    ts.isNoSubstitutionTemplateLiteral(name) ||
    ts.isNumericLiteral(name)
  ) {
    return name.text;
  }

  return null;
}

function unwrapExpression(expression) {
  if (
    ts.isAsExpression(expression) ||
    ts.isParenthesizedExpression(expression) ||
    ts.isTypeAssertionExpression(expression) ||
    (typeof ts.isSatisfiesExpression === 'function' && ts.isSatisfiesExpression(expression))
  ) {
    return unwrapExpression(expression.expression);
  }

  return expression;
}

function collectObjectLiteralKeys(expression, variables, visited = new Set()) {
  const keys = new Set();
  const target = unwrapExpression(expression);

  if (!target || visited.has(target)) {
    return keys;
  }

  visited.add(target);

  if (!ts.isObjectLiteralExpression(target)) {
    return keys;
  }

  for (const property of target.properties) {
    if (ts.isPropertyAssignment(property) || ts.isShorthandPropertyAssignment(property)) {
      const key = getPropertyNameText(property.name);
      if (key) {
        keys.add(key);
      }
      continue;
    }

    if (ts.isSpreadAssignment(property) && ts.isIdentifier(property.expression)) {
      const spreadTarget = variables.get(property.expression.text);
      if (spreadTarget) {
        for (const key of collectObjectLiteralKeys(spreadTarget, variables, visited)) {
          keys.add(key);
        }
      }
    }
  }

  return keys;
}

function collectZhCatalogKeys() {
  const translationModule = loadTsModule(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-admin-core',
      'src',
      'i18nTranslations.ts',
    ),
  );
  const catalog = translationModule?.ADMIN_TRANSLATIONS?.['zh-CN'];
  assert.ok(catalog && typeof catalog === 'object');
  return new Set(Object.keys(catalog));
}

function collectPropertyStringValues(relativePath, propertyNames) {
  const source = read(relativePath);
  const sourceFile = ts.createSourceFile(
    relativePath,
    source,
    ts.ScriptTarget.Latest,
    true,
    relativePath.endsWith('.tsx') ? ts.ScriptKind.TSX : ts.ScriptKind.TS,
  );
  const values = new Set();

  function visit(node) {
    if (
      ts.isPropertyAssignment(node) &&
      getPropertyNameText(node.name) &&
      propertyNames.has(getPropertyNameText(node.name))
    ) {
      const expression = unwrapExpression(node.initializer);
      if (
        (ts.isStringLiteral(expression) || ts.isNoSubstitutionTemplateLiteral(expression)) &&
        hasVisibleTextCandidate(expression.text)
      ) {
        values.add(expression.text);
      }
    }

    ts.forEachChild(node, visit);
  }

  visit(sourceFile);

  return [...values].sort((left, right) => left.localeCompare(right));
}

test('admin core owns locale options and i18n helpers without the legacy commons package', () => {
  const coreIndex = read('packages/sdkwork-router-admin-core/src/index.tsx');
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');

  assert.match(coreIndex, /AdminI18nProvider/);
  assert.match(coreIndex, /useAdminI18n/);
  assert.match(i18n, /ADMIN_LOCALE_OPTIONS/);
  assert.match(i18n, /'en-US'/);
  assert.match(i18n, /'zh-CN'/);
  assert.match(i18n, /ADMIN_TRANSLATIONS/);
  assert.match(i18n, /translateAdminText/);
  assert.match(i18n, /formatAdminDateTime/);
  assert.match(i18n, /formatAdminNumber/);
  assert.match(i18n, /formatAdminCurrency/);
  assert.doesNotMatch(i18n, /sdkwork-router-admin-commons/);
});

test('all admin source files keep visible JSX copy behind admin i18n helpers', () => {
  const coverage = listAdminSourceFiles().filter(
    (relativePath) => relativePath !== 'packages/sdkwork-router-admin-core/src/i18n.tsx',
  );

  for (const relativePath of coverage) {
    assert.deepEqual(collectVisibleTextCandidates(relativePath), [], relativePath);
  }
});

test('all admin translation keys used in source are present in the zh-CN catalog', () => {
  const usedKeys = collectUsedTranslationKeys();
  const catalogKeys = collectZhCatalogKeys();
  const missingKeys = usedKeys.filter((key) => !catalogKeys.has(key));

  assert.deepEqual(missingKeys, []);
});

test('dynamic metadata translation keys used by routes, service plans, and config option sources are present in the zh-CN catalog', () => {
  const catalogKeys = collectZhCatalogKeys();
  const dynamicMetadataSources = [
    {
      relativePath: 'packages/sdkwork-router-admin-auth/src/index.tsx',
      propertyNames: new Set([
        'alternateLabel',
        'description',
        'modeLabel',
        'submitLabel',
        'title',
      ]),
    },
    {
      relativePath: 'packages/sdkwork-router-admin-apirouter/src/services/gatewayApiKeyAccessService.ts',
      propertyNames: new Set(['applyLabel', 'description', 'label', 'title']),
    },
    {
      relativePath: 'packages/sdkwork-router-admin-apirouter/src/pages/access/shared.ts',
      propertyNames: new Set(['codex', 'claude-code', 'opencode', 'gemini', 'openclaw']),
    },
    {
      relativePath: 'packages/sdkwork-router-admin-catalog/src/page/shared.tsx',
      propertyNames: new Set(['label']),
    },
    {
      relativePath: 'packages/sdkwork-router-admin-core/src/routes.ts',
      propertyNames: new Set(['detail', 'eyebrow', 'group', 'label']),
    },
    {
      relativePath: 'packages/sdkwork-router-admin-core/src/workbenchSnapshot.ts',
      propertyNames: new Set(['detail', 'label', 'title']),
    },
    {
      relativePath: 'packages/sdkwork-router-admin-settings/src/AppearanceSettings.tsx',
      propertyNames: new Set(['label']),
    },
    {
      relativePath: 'packages/sdkwork-router-admin-traffic/src/index.tsx',
      propertyNames: new Set(['label']),
    },
  ];

  const dynamicKeys = dynamicMetadataSources.flatMap(({ relativePath, propertyNames }) =>
    collectPropertyStringValues(relativePath, propertyNames),
  );
  const missingKeys = [...new Set(dynamicKeys)].filter((key) => !catalogKeys.has(key));

  assert.deepEqual(missingKeys, []);
});

test('dynamic metadata sources flow through translated consumption boundaries before they render in admin UI', () => {
  const authPage = read('packages/sdkwork-router-admin-auth/src/index.tsx');
  const apiKeyUsageDialog = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayApiKeyUsageDialog.tsx',
  );
  const gatewayAccessShared = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/shared.ts',
  );
  const catalogShared = read('packages/sdkwork-router-admin-catalog/src/page/shared.tsx');
  const overviewPage = read('packages/sdkwork-router-admin-overview/src/index.tsx');
  const appearanceSettings = read('packages/sdkwork-router-admin-settings/src/AppearanceSettings.tsx');
  const navigationSettings = read('packages/sdkwork-router-admin-settings/src/NavigationSettings.tsx');
  const sidebar = read('packages/sdkwork-router-admin-shell/src/components/Sidebar.tsx');
  const trafficPage = read('packages/sdkwork-router-admin-traffic/src/index.tsx');

  assert.match(authPage, /t\(copy\.title\)/);
  assert.match(authPage, /t\(copy\.description\)/);
  assert.match(authPage, /t\(copy\.modeLabel\)/);
  assert.match(authPage, /t\([^)]*copy\.submitLabel[^)]*\)/);
  assert.match(authPage, /t\(copy\.alternateLabel\)/);

  assert.match(apiKeyUsageDialog, /t\(selectedPlan\.description\)/);
  assert.match(apiKeyUsageDialog, /t\(selectedPlan\.label\)/);
  assert.match(apiKeyUsageDialog, /t\(snippet\.title\)/);
  assert.match(apiKeyUsageDialog, /t\(QUICK_SETUP_CLIENT_LABELS\[plan\.id\] \?\? plan\.label\)/);

  assert.match(gatewayAccessShared, /translateAdminText\('Applied setup to \{count\} OpenClaw instance\(s\)\.'/);
  assert.match(catalogShared, /priceUnitLabel\(value: string\)[\s\S]*translateAdminText\(/);
  assert.match(overviewPage, /t\(metric\.detail\)/);
  assert.match(overviewPage, /t\(metric\.label\)/);
  assert.match(overviewPage, /t\(alert\.title\)/);
  assert.match(overviewPage, /t\(alert\.detail\)/);
  assert.match(appearanceSettings, /t\(color\.label\)/);
  assert.match(navigationSettings, /t\(route\.label\)/);
  assert.match(navigationSettings, /t\(route\.detail\)/);
  assert.match(navigationSettings, /t\(route\.group \?\? 'Workspace'\)/);
  assert.match(sidebar, /label: t\(route\.label\)/);
  assert.match(sidebar, /section: t\(section\)/);
  assert.match(trafficPage, /viewModeOptions\.map\(\(option\) => \(\{ \.\.\.option, label: t\(option\.label\) \}\)\)/);
  assert.match(trafficPage, /recentWindowOptions\.map\(\(option\) => \(\{ \.\.\.option, label: t\(option\.label\) \}\)\)/);
});
