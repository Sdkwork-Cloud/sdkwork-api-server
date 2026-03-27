import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

test('router product service wrapper strips pnpm forwarding separator before passing CLI args to cargo', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'run-router-product-service.mjs')).href
  );

  assert.deepEqual(module.normalizeRouterProductServiceArgs(['--', '--help']), ['--help']);
  assert.deepEqual(
    module.normalizeRouterProductServiceArgs(['--', '--roles', 'web']),
    ['--roles', 'web'],
  );
  assert.deepEqual(
    module.normalizeRouterProductServiceArgs([
      '--dry-run',
      '--plan-format',
      'json',
      '--',
      '--roles',
      'web',
    ]),
    ['--dry-run', '--plan-format', 'json', '--roles', 'web'],
  );
  assert.deepEqual(
    module.normalizeRouterProductServiceArgs(['--bind', '0.0.0.0:3001']),
    ['--bind', '0.0.0.0:3001'],
  );
});
