#!/usr/bin/env node

import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import { brotliCompressSync, constants as zlibConstants, gzipSync } from 'node:zlib';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const vendorChunkPrefixes = [
  'react-vendor',
  'radix-vendor',
  'style-vendor',
  'charts-vendor',
  'icon-vendor',
  'motion-vendor',
];

export const defaultFrontendBudgetRules = [
  {
    ruleId: 'admin-entry-js',
    appDir: path.join('apps', 'sdkwork-router-admin'),
    description: 'admin entry JavaScript',
    selector: 'entry-js',
    budgets: {
      raw: 300_000,
      gzip: 90_000,
    },
  },
  {
    ruleId: 'admin-entry-css',
    appDir: path.join('apps', 'sdkwork-router-admin'),
    description: 'admin entry stylesheet',
    selector: 'entry-css',
    budgets: {
      raw: 130_000,
      gzip: 23_000,
    },
  },
  {
    ruleId: 'admin-react-vendor',
    appDir: path.join('apps', 'sdkwork-router-admin'),
    description: 'admin React vendor chunk',
    selector: {
      type: 'pattern',
      pattern: /^react-vendor.*\.js$/i,
    },
    budgets: {
      raw: 250_000,
      gzip: 80_000,
    },
  },
  {
    ruleId: 'admin-max-lazy-js',
    appDir: path.join('apps', 'sdkwork-router-admin'),
    description: 'admin largest async JavaScript chunk',
    selector: 'largest-non-vendor-js',
    budgets: {
      raw: 180_000,
      gzip: 45_000,
    },
  },
  {
    ruleId: 'portal-entry-js',
    appDir: path.join('apps', 'sdkwork-router-portal'),
    description: 'portal entry JavaScript',
    selector: 'entry-js',
    budgets: {
      raw: 235_000,
      gzip: 80_000,
    },
  },
  {
    ruleId: 'portal-entry-css',
    appDir: path.join('apps', 'sdkwork-router-portal'),
    description: 'portal entry stylesheet',
    selector: 'entry-css',
    budgets: {
      raw: 185_000,
      gzip: 30_000,
    },
  },
  {
    ruleId: 'portal-react-vendor',
    appDir: path.join('apps', 'sdkwork-router-portal'),
    description: 'portal React vendor chunk',
    selector: {
      type: 'pattern',
      pattern: /^react-vendor.*\.js$/i,
    },
    budgets: {
      raw: 270_000,
      gzip: 85_000,
    },
  },
  {
    ruleId: 'portal-locale-catalog-js',
    appDir: path.join('apps', 'sdkwork-router-portal'),
    description: 'portal zh-CN locale catalog chunk',
    selector: {
      type: 'pattern',
      pattern: /^portalMessages\.zh-CN.*\.js$/i,
    },
    budgets: {
      raw: 135_000,
      gzip: 48_000,
    },
  },
  {
    ruleId: 'portal-max-lazy-js',
    appDir: path.join('apps', 'sdkwork-router-portal'),
    description: 'portal largest async JavaScript chunk',
    selector: {
      type: 'largest-non-vendor-js',
      excludePatterns: [/^portalMessages\.zh-CN.*\.js$/i],
    },
    budgets: {
      raw: 100_000,
      gzip: 25_000,
    },
  },
];

function toDisplayPath(value) {
  return value.replaceAll('\\', '/');
}

function formatBytes(bytes) {
  return `${(bytes / 1024).toFixed(1)} KiB`;
}

function normalizeAssetReference(reference) {
  const cleanPath = reference.split('?')[0].split('#')[0];
  return path.posix.basename(cleanPath.replaceAll('\\', '/'));
}

function parseIndexAssetReferences(indexHtml) {
  const entryScripts = [];
  const entryStylesheets = [];

  for (const match of indexHtml.matchAll(/<script\b[^>]*\bsrc="([^"]+)"/gi)) {
    entryScripts.push(normalizeAssetReference(match[1]));
  }

  for (const match of indexHtml.matchAll(/<link\b[^>]*\bhref="([^"]+)"[^>]*>/gi)) {
    const tag = match[0];
    if (!/\brel="stylesheet"/i.test(tag)) {
      continue;
    }
    entryStylesheets.push(normalizeAssetReference(match[1]));
  }

  return {
    entryScripts,
    entryStylesheets,
  };
}

function measureAsset(assetPath) {
  const source = readFileSync(assetPath);
  return {
    name: path.basename(assetPath),
    path: assetPath,
    raw: source.length,
    gzip: gzipSync(source, { level: 9 }).length,
    brotli: brotliCompressSync(source, {
      params: {
        [zlibConstants.BROTLI_PARAM_QUALITY]: 11,
      },
    }).length,
  };
}

function loadAppInventory(appRoot) {
  const indexHtmlPath = path.join(appRoot, 'dist', 'index.html');
  const assetsRoot = path.join(appRoot, 'dist', 'assets');

  if (!existsSync(indexHtmlPath)) {
    throw new Error(`missing frontend entrypoint ${toDisplayPath(indexHtmlPath)}`);
  }
  if (!existsSync(assetsRoot)) {
    throw new Error(`missing frontend asset directory ${toDisplayPath(assetsRoot)}`);
  }

  const assets = readdirSync(assetsRoot)
    .map((name) => path.join(assetsRoot, name))
    .filter((candidatePath) => statSync(candidatePath).isFile())
    .map((candidatePath) => measureAsset(candidatePath));

  const indexHtml = readFileSync(indexHtmlPath, 'utf8');
  const references = parseIndexAssetReferences(indexHtml);
  const byName = new Map(assets.map((asset) => [asset.name, asset]));

  return {
    appRoot,
    assets,
    entryScriptAssets: references.entryScripts
      .map((name) => byName.get(name))
      .filter(Boolean),
    entryStylesheetAssets: references.entryStylesheets
      .map((name) => byName.get(name))
      .filter(Boolean),
  };
}

function isVendorChunk(assetName) {
  return vendorChunkPrefixes.some((prefix) => assetName.startsWith(prefix));
}

function selectLargestAsset(candidates) {
  return [...candidates].sort((left, right) => right.raw - left.raw)[0] ?? null;
}

function selectLargestNonVendorJsAsset(inventory, excludePatterns = []) {
  return selectLargestAsset(
    inventory.assets.filter((asset) => (
      asset.name.endsWith('.js')
      && !inventory.entryScriptAssets.some((entryAsset) => entryAsset.name === asset.name)
      && !isVendorChunk(asset.name)
      && !excludePatterns.some((pattern) => pattern.test(asset.name))
    )),
  );
}

function selectRuleAsset(rule, inventory) {
  if (rule.selector === 'entry-js') {
    return selectLargestAsset(inventory.entryScriptAssets);
  }

  if (rule.selector === 'entry-css') {
    return selectLargestAsset(inventory.entryStylesheetAssets);
  }

  if (rule.selector === 'largest-non-vendor-js') {
    return selectLargestNonVendorJsAsset(inventory);
  }

  if (rule.selector?.type === 'largest-non-vendor-js') {
    return selectLargestNonVendorJsAsset(
      inventory,
      rule.selector.excludePatterns ?? [],
    );
  }

  if (rule.selector?.type === 'pattern') {
    return selectLargestAsset(
      inventory.assets.filter((asset) => rule.selector.pattern.test(asset.name)),
    );
  }

  throw new Error(`unsupported frontend budget selector ${JSON.stringify(rule.selector)}`);
}

function evaluateRule(rule, inventory) {
  const asset = selectRuleAsset(rule, inventory);

  if (!asset) {
    return {
      ok: false,
      ruleId: rule.ruleId,
      appDir: rule.appDir,
      description: rule.description,
      reason: 'missing-asset',
      message: `${rule.description} was not found in ${toDisplayPath(path.join(rule.appDir, 'dist', 'assets'))}`,
    };
  }

  const overages = Object.entries(rule.budgets)
    .filter(([metricName, limit]) => asset[metricName] > limit)
    .map(([metricName, limit]) => ({
      metricName,
      actual: asset[metricName],
      limit,
    }));

  return {
    ok: overages.length === 0,
    ruleId: rule.ruleId,
    appDir: rule.appDir,
    description: rule.description,
    asset,
    overages,
    message: overages.length === 0
      ? `${rule.description} (${asset.name}) raw=${formatBytes(asset.raw)} gzip=${formatBytes(asset.gzip)}`
      : `${rule.description} (${asset.name}) raw=${formatBytes(asset.raw)} gzip=${formatBytes(asset.gzip)} exceeded ${overages.map((overage) => `${overage.metricName}<=${formatBytes(overage.limit)}`).join(', ')}`,
  };
}

export function evaluateFrontendBudgets({
  workspaceRoot = path.resolve(__dirname, '..'),
  rules = defaultFrontendBudgetRules,
} = {}) {
  const inventoryByAppDir = new Map();

  for (const rule of rules) {
    if (inventoryByAppDir.has(rule.appDir)) {
      continue;
    }
    inventoryByAppDir.set(
      rule.appDir,
      loadAppInventory(path.join(workspaceRoot, rule.appDir)),
    );
  }

  const results = rules.map((rule) => evaluateRule(rule, inventoryByAppDir.get(rule.appDir)));
  const failures = results.filter((result) => !result.ok);
  const summaryLines = [
    ...results.map((result) => `${result.ok ? 'PASS' : 'FAIL'} ${result.message}`),
  ];

  return {
    ok: failures.length === 0,
    results,
    failures,
    summary: summaryLines.join('\n'),
  };
}

export function assertFrontendBudgets(options = {}) {
  const report = evaluateFrontendBudgets(options);
  if (!report.ok) {
    throw new Error(report.summary);
  }
  return report;
}

function main() {
  const report = assertFrontendBudgets();
  console.error(report.summary);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    main();
  } catch (error) {
    console.error(`[check-router-frontend-budgets] ${error.message}`);
    process.exit(1);
  }
}
