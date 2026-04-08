import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const workspaceRoot = path.resolve(import.meta.dirname, '..', '..', '..');

test('workspace patches zip away from zlib-rs for Windows rust verification stability', () => {
  const workspaceCargoToml = fs.readFileSync(path.join(workspaceRoot, 'Cargo.toml'), 'utf8');
  assert.match(
    workspaceCargoToml,
    /\[patch\.crates-io\][\s\S]*zip\s*=\s*\{\s*path\s*=\s*"vendor\/zip-3\.0\.0"\s*\}/,
  );

  const vendoredZipCargoToml = fs.readFileSync(
    path.join(workspaceRoot, 'vendor', 'zip-3.0.0', 'Cargo.toml'),
    'utf8',
  );

  const deflateFeature = vendoredZipCargoToml.match(/deflate = \[(?<body>[\s\S]*?)\n\]/);
  assert.ok(deflateFeature?.groups?.body, 'expected vendored zip deflate feature definition');
  assert.match(deflateFeature.groups.body, /deflate-zopfli/);
  assert.match(deflateFeature.groups.body, /deflate-flate2/);
  assert.match(deflateFeature.groups.body, /flate2\/rust_backend/);
  assert.doesNotMatch(deflateFeature.groups.body, /deflate-flate2-zlib-rs/);
});
