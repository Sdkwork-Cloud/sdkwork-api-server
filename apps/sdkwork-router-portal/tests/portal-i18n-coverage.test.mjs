import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import ts from 'typescript';

const appRoot = path.resolve(import.meta.dirname, '..');
const packagesRoot = path.join(appRoot, 'packages');
const commonsPath = path.join(packagesRoot, 'sdkwork-router-portal-commons', 'src', 'index.tsx');
const NON_TRANSLATABLE_KEYS = new Set(['SPRING20']);
const PAGE_COMPONENT_PATH_PATTERN = /[\\/]src[\\/](pages|components)[\\/].+\.(ts|tsx)$/;
const UI_SHELL_PATH_PATTERN = /[\\/]src[\\/](application|lib|store)[\\/].+\.(ts|tsx)$/;
const ROUTE_CONFIG_PATH_PATTERN = /[\\/]src[\\/]routes\.ts$/;
const SERVICE_PATH_PATTERN = /[\\/]src[\\/]services[\\/].+\.(ts|tsx)$/;
const VISIBLE_JSX_PROPS = new Set([
  'aria-label',
  'changeLabel',
  'description',
  'eyebrow',
  'label',
  'placeholder',
  'summary',
  'title',
]);
const VISIBLE_OBJECT_FIELDS = new Set([
  'action_label',
  'changeLabel',
  'cta',
  'description',
  'detail',
  'emptyDetail',
  'emptyTitle',
  'eyebrow',
  'headline',
  'label',
  'placeholder',
  'primaryLabel',
  'secondaryLabel',
  'status',
  'statusLabel',
  'status_label',
  'summary',
  'title',
]);
const ALLOWED_RAW_TEXT = new Set([
  'GitHub',
  'Google',
  'WELCOME100',
  'TEAMREADY',
  'OpenAI',
  'Anthropic',
  'Gemini',
  'Codex',
  'Claude Code',
  'OpenCode',
  'OpenClaw',
  'generateContent',
  'us-east',
  'us-west',
  'eu-west',
  'ap-southeast',
  'skw_live_custom_portal_secret',
  'predictable',
  '0.30',
  '250',
  'chat_completion',
  '-&gt;',
  'minimizeWindow',
  'maximizeWindow',
  'closeWindow',
]);
const VISIBLE_VARIABLE_NAME_PATTERN =
  /(status|title|description|headline|label|detail|summary|message|copy|placeholder|eyebrow)$/i;
const VISIBLE_FUNCTION_NAME_PATTERN =
  /(Label|Title|Detail|Description|Headline|Message|Copy|Status|Text|Summary)$/;
const VISIBLE_OWNER_NAME_PATTERN = /Labels$/;
const VISIBLE_SETTER_NAME_PATTERN =
  /^set(Status|Title|Description|Message|Label|BootstrapStatus|DashboardStatus)$/;

function walkFiles(dir, output = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === 'node_modules' || entry.name === 'dist' || entry.name === 'tests') {
        continue;
      }
      walkFiles(fullPath, output);
      continue;
    }

    if (/\.(ts|tsx)$/.test(entry.name)) {
      output.push(fullPath);
    }
  }

  return output;
}

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function nodeText(value) {
  if (!value) {
    return '';
  }

  if (ts.isStringLiteralLike(value)) {
    return value.text;
  }

  return value.getText().replace(/^`|`$/g, '');
}

function normalizeText(text) {
  return text.replace(/\s+/g, ' ').trim();
}

function escapeRegExp(text) {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function isIgnoredRawText(text) {
  const normalized = normalizeText(text);

  if (!normalized) {
    return true;
  }

  if (ALLOWED_RAW_TEXT.has(normalized)) {
    return true;
  }

  if (/^[,.:;()[\]{}<>&/+*-]+$/.test(normalized)) {
    return true;
  }

  if (/^(https?:\/\/|\/v\d|pnpm |curl |sdkwork-router-portal$)/.test(normalized)) {
    return true;
  }

  if (/^[A-Z0-9_-]+$/.test(normalized)) {
    return true;
  }

  return false;
}

function isTranslationCall(node) {
  return ts.isCallExpression(node)
    && ts.isIdentifier(node.expression)
    && (node.expression.text === 't' || node.expression.text === 'translatePortalText');
}

function hasTranslationAncestor(node) {
  let current = node.parent;

  while (current) {
    if (isTranslationCall(current)) {
      return true;
    }
    current = current.parent;
  }

  return false;
}

function propertyName(node) {
  if (ts.isIdentifier(node) || ts.isStringLiteralLike(node)) {
    return node.text;
  }

  return null;
}

function functionName(node) {
  let current = node.parent;

  while (current) {
    if (
      (ts.isFunctionDeclaration(current) || ts.isMethodDeclaration(current) || ts.isFunctionExpression(current))
      && current.name
      && ts.isIdentifier(current.name)
    ) {
      return current.name.text;
    }

    if (ts.isArrowFunction(current) && current.parent && ts.isVariableDeclaration(current.parent)) {
      const name = current.parent.name;
      if (ts.isIdentifier(name)) {
        return name.text;
      }
    }

    current = current.parent;
  }

  return null;
}

function ownerVariableName(node) {
  let current = node.parent;

  while (current) {
    if (ts.isVariableDeclaration(current) && ts.isIdentifier(current.name)) {
      return current.name.text;
    }
    current = current.parent;
  }

  return null;
}

function collectRawUserFacingStrings(file) {
  const source = readFileSync(file, 'utf8');
  const sourceFile = ts.createSourceFile(file, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
  const findings = [];
  const scanUi =
    PAGE_COMPONENT_PATH_PATTERN.test(file)
    || UI_SHELL_PATH_PATTERN.test(file)
    || ROUTE_CONFIG_PATH_PATTERN.test(file)
    || file === commonsPath;
  const scanService = SERVICE_PATH_PATTERN.test(file);

  function push(node, text, kind) {
    const normalized = normalizeText(text);

    if (isIgnoredRawText(normalized) || hasTranslationAncestor(node)) {
      return;
    }

    const line = sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile)).line + 1;
    findings.push({
      file: path.relative(appRoot, file),
      line,
      kind,
      text: normalized,
    });
  }

  function visit(node) {
    if (scanUi && ts.isJsxText(node)) {
      push(node, node.getText(sourceFile), 'jsx');
    }

    if (scanUi && ts.isJsxAttribute(node) && VISIBLE_JSX_PROPS.has(node.name.text) && node.initializer) {
      if (ts.isStringLiteralLike(node.initializer)) {
        push(node.initializer, node.initializer.text, `prop:${node.name.text}`);
      } else if (
        ts.isJsxExpression(node.initializer)
        && node.initializer.expression
        && (ts.isStringLiteralLike(node.initializer.expression)
          || ts.isNoSubstitutionTemplateLiteral(node.initializer.expression)
          || ts.isTemplateExpression(node.initializer.expression))
      ) {
        push(
          node.initializer.expression,
          nodeText(node.initializer.expression),
          `prop:${node.name.text}`,
        );
      }
    }

    if ((scanUi || scanService) && ts.isPropertyAssignment(node)) {
      const name = propertyName(node.name);
      if (
        name
        && VISIBLE_OBJECT_FIELDS.has(name)
        && (ts.isStringLiteralLike(node.initializer)
          || ts.isNoSubstitutionTemplateLiteral(node.initializer)
          || ts.isTemplateExpression(node.initializer))
      ) {
        push(node.initializer, nodeText(node.initializer), `field:${name}`);
      } else {
        const ownerName = ownerVariableName(node);
        if (
          scanUi
          && ownerName
          && VISIBLE_OWNER_NAME_PATTERN.test(ownerName)
          && (ts.isStringLiteralLike(node.initializer)
            || ts.isNoSubstitutionTemplateLiteral(node.initializer)
            || ts.isTemplateExpression(node.initializer))
        ) {
          push(node.initializer, nodeText(node.initializer), `owner:${ownerName}`);
        }
      }
    }

    if ((scanUi || scanService) && ts.isVariableDeclaration(node) && ts.isIdentifier(node.name)) {
      if (
        VISIBLE_VARIABLE_NAME_PATTERN.test(node.name.text)
        && node.initializer
        && (ts.isStringLiteralLike(node.initializer)
          || ts.isNoSubstitutionTemplateLiteral(node.initializer)
          || ts.isTemplateExpression(node.initializer))
      ) {
        push(node.initializer, nodeText(node.initializer), `variable:${node.name.text}`);
      }
    }

    if ((scanUi || scanService) && ts.isVariableDeclaration(node) && ts.isArrayBindingPattern(node.name)) {
      const [firstBinding] = node.name.elements;
      if (
        firstBinding
        && ts.isBindingElement(firstBinding)
        && ts.isIdentifier(firstBinding.name)
        && VISIBLE_VARIABLE_NAME_PATTERN.test(firstBinding.name.text)
        && node.initializer
        && ts.isCallExpression(node.initializer)
        && ts.isIdentifier(node.initializer.expression)
        && node.initializer.expression.text === 'useState'
        && node.initializer.arguments[0]
        && (ts.isStringLiteralLike(node.initializer.arguments[0])
          || ts.isNoSubstitutionTemplateLiteral(node.initializer.arguments[0])
          || ts.isTemplateExpression(node.initializer.arguments[0]))
      ) {
        push(
          node.initializer.arguments[0],
          nodeText(node.initializer.arguments[0]),
          `state:${firstBinding.name.text}`,
        );
      }
    }

    if ((scanUi || scanService) && ts.isCallExpression(node) && ts.isIdentifier(node.expression)) {
      if (
        VISIBLE_SETTER_NAME_PATTERN.test(node.expression.text)
        && node.arguments[0]
        && (ts.isStringLiteralLike(node.arguments[0])
          || ts.isNoSubstitutionTemplateLiteral(node.arguments[0])
          || ts.isTemplateExpression(node.arguments[0]))
      ) {
        push(node.arguments[0], nodeText(node.arguments[0]), `call:${node.expression.text}`);
      }
    }

    if ((scanUi || scanService) && ts.isReturnStatement(node) && node.expression) {
      const ownerName = functionName(node);
      if (
        ownerName
        && VISIBLE_FUNCTION_NAME_PATTERN.test(ownerName)
        && (ts.isStringLiteralLike(node.expression)
          || ts.isNoSubstitutionTemplateLiteral(node.expression)
          || ts.isTemplateExpression(node.expression))
      ) {
        push(node.expression, nodeText(node.expression), `return:${ownerName}`);
      }
    }

    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  return findings;
}

test('portal pages, shell files, and services avoid raw user-facing strings', () => {
  const files = walkFiles(packagesRoot).filter((file) => (
    PAGE_COMPONENT_PATH_PATTERN.test(file)
    || UI_SHELL_PATH_PATTERN.test(file)
    || ROUTE_CONFIG_PATH_PATTERN.test(file)
    || SERVICE_PATH_PATTERN.test(file)
    || file === commonsPath
  ));
  const findings = files.flatMap((file) => collectRawUserFacingStrings(file));

  assert.deepEqual(findings, []);
});

test('portal shared locale dictionaries expose real Simplified Chinese translations for shell copy', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const portalCore = read('packages/sdkwork-router-portal-commons/src/i18n-core.ts');
  const zhMessages = read('packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts');
  const criticalOperatorKeys = [
    'Access and onboarding',
    'Access readiness',
    'Admin bind',
    'All environments',
    'Anthropic Messages',
    'Anthropic Messages route check',
    'Apply setup',
    'Auth',
    'Cash balance',
    'Closed checkout',
    'Command workbench',
    'Commerce posture',
    'Commercial runway',
    'Compatibility routes',
    'Config',
    'Desktop bridge',
    'Desktop embedded runtime',
    'Gateway base',
    'Gateway bind',
    'Gemini generateContent route check',
    'How to use this key',
    'Ledger evidence is waiting for financial activity',
    'Ledger evidence should drive money decisions',
    'OpenAI-compatible route check',
    'Product entrypoint',
    'Protocol families',
    'Provider manifest',
    'Settings',
    'Unassigned',
    'Usage method',
  ];

  assert.doesNotMatch(commons, /Object\.fromEntries\(PORTAL_MESSAGE_KEYS\.map/);
  assert.match(commons, /'Developer portal'/);
  assert.match(commons, /'Simplified Chinese'/);
  assert.match(commons, /'Choose the portal workspace language\.[^']*'/);
  assert.match(
    portalCore,
    /runtimePortalMessagesByLocale\[locale\]\[text\][\s\S]*runtimePortalFallbackByLocale\[locale\]\?\.?\(text\)[\s\S]*\?\? text/,
  );
  assert.match(zhMessages, /'Predictable order': '[^']+'/);
  assert.match(zhMessages, /'Developer portal': '[^']+'/);
  assert.match(zhMessages, /'Simplified Chinese': '[^']+'/);
  for (const key of criticalOperatorKeys) {
    assert.match(zhMessages, new RegExp(`'${escapeRegExp(key)}': '[^']+'`));
  }
  assert.match(zhMessages, /[\u4e00-\u9fff]/);
  assert.doesNotMatch(
    zhMessages,
    /\u00e6\u00b0\u201c|\u00e5\u00bf\u2122|\u00e8\u0152\u2026|\u00e8\u017d\u00bd/,
  );
});
