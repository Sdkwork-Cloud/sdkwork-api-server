#!/usr/bin/env node

import { existsSync, readdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export const DOC_BOOTSTRAP_SCAN_ROOTS = [
  'docs/getting-started',
  'docs/api-reference',
  'docs/operations',
  'docs/zh/getting-started',
  'docs/zh/api-reference',
];

export const DOC_BOOTSTRAP_SCAN_FILES = [
  'README.md',
  'README.zh-CN.md',
];

export const RETIRED_BOOTSTRAP_MARKERS = [
  {
    name: 'retired admin bootstrap email',
    pattern: /admin@sdkwork\.local/i,
  },
  {
    name: 'retired portal bootstrap email',
    pattern: /portal@sdkwork\.local/i,
  },
  {
    name: 'retired bootstrap password',
    pattern: /ChangeMe123!/i,
  },
  {
    name: 'retired seeded credentials copy',
    pattern: /Seeded local credentials/i,
  },
  {
    name: 'retired default operator copy',
    pattern: /default local operator/i,
  },
  {
    name: 'retired demo account copy',
    pattern: /demo local account/i,
  },
  {
    name: 'retired built-in demo account copy',
    pattern: /built-in .*demo accounts/i,
  },
];

function listMarkdownFiles(rootDir) {
  if (!existsSync(rootDir)) {
    return [];
  }

  const files = [];
  for (const entry of readdirSync(rootDir, { withFileTypes: true })) {
    const absolutePath = path.join(rootDir, entry.name);
    if (entry.isDirectory()) {
      files.push(...listMarkdownFiles(absolutePath));
      continue;
    }
    if (entry.isFile() && absolutePath.endsWith('.md')) {
      files.push(absolutePath);
    }
  }
  return files;
}

export function scanDocsForRetiredBootstrapCredentials({
  workspaceRoot = path.resolve(__dirname, '..'),
} = {}) {
  const findings = [];

  for (const relativeFile of DOC_BOOTSTRAP_SCAN_FILES) {
    const absolutePath = path.join(workspaceRoot, relativeFile);
    if (!existsSync(absolutePath)) {
      continue;
    }

    const lines = readFileSync(absolutePath, 'utf8').split(/\r?\n/u);
    for (let index = 0; index < lines.length; index += 1) {
      const line = lines[index];
      for (const marker of RETIRED_BOOTSTRAP_MARKERS) {
        if (!marker.pattern.test(line)) {
          continue;
        }
        findings.push({
          file: path.relative(workspaceRoot, absolutePath).replace(/\\/gu, '/'),
          line: index + 1,
          marker: marker.name,
          excerpt: line.trim(),
        });
      }
    }
  }

  for (const relativeRoot of DOC_BOOTSTRAP_SCAN_ROOTS) {
    const absoluteRoot = path.join(workspaceRoot, relativeRoot);
    for (const filePath of listMarkdownFiles(absoluteRoot)) {
      const lines = readFileSync(filePath, 'utf8').split(/\r?\n/u);
      for (let index = 0; index < lines.length; index += 1) {
        const line = lines[index];
        for (const marker of RETIRED_BOOTSTRAP_MARKERS) {
          if (!marker.pattern.test(line)) {
            continue;
          }
          findings.push({
            file: path.relative(workspaceRoot, filePath).replace(/\\/gu, '/'),
            line: index + 1,
            marker: marker.name,
            excerpt: line.trim(),
          });
        }
      }
    }
  }

  return findings.sort((left, right) => (
    left.file.localeCompare(right.file)
    || left.line - right.line
    || left.marker.localeCompare(right.marker)
  ));
}

function main() {
  const findings = scanDocsForRetiredBootstrapCredentials();
  if (findings.length === 0) {
    return;
  }

  console.error('[check-router-docs-safety] Retired bootstrap credential markers found:');
  for (const finding of findings) {
    console.error(
      `[check-router-docs-safety] ${finding.file}:${finding.line} ${finding.marker}: ${finding.excerpt}`,
    );
  }
  process.exit(1);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main();
}
