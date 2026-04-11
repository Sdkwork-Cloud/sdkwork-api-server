import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('routing profile dialog uses canonical backend routing strategy values', () => {
  const dialog = read('packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutingProfilesDialog.tsx');

  assert.match(dialog, /deterministic_priority/);
  assert.doesNotMatch(dialog, /strategy:\s*'priority'/);
  assert.doesNotMatch(dialog, /value:\s*'priority'/);
  assert.doesNotMatch(dialog, /value:\s*'balanced'/);
  assert.doesNotMatch(dialog, /value:\s*'latency_optimized'/);
  assert.doesNotMatch(dialog, /value:\s*'cost_optimized'/);
});
