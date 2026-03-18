import type { CreatedGatewayApiKey, GatewayApiKeyRecord } from 'sdkwork-router-portal-types';

import type {
  ApiKeyEnvironmentStrategyItem,
  ApiKeyEnvironmentSummary,
  ApiKeyGuardrail,
  ApiKeyRotationStep,
  PortalApiKeysPageViewModel,
} from '../types';

const environmentOrder = ['test', 'staging', 'live'];

function summarizeKeysByEnvironment(keys: GatewayApiKeyRecord[]): ApiKeyEnvironmentSummary[] {
  const grouped = new Map<string, ApiKeyEnvironmentSummary>();

  for (const key of keys) {
    const current = grouped.get(key.environment) ?? {
      environment: key.environment,
      total: 0,
      active: 0,
    };

    current.total += 1;
    if (key.active) {
      current.active += 1;
    }
    grouped.set(key.environment, current);
  }

  return [...grouped.values()].sort((left, right) => left.environment.localeCompare(right.environment));
}

function buildEnvironmentStrategy(keys: GatewayApiKeyRecord[]): ApiKeyEnvironmentStrategyItem[] {
  const grouped = new Map<string, ApiKeyEnvironmentSummary>();

  for (const summary of summarizeKeysByEnvironment(keys)) {
    grouped.set(summary.environment, summary);
  }

  const recommendedEnvironment = environmentOrder.find((environment) => {
    const summary = grouped.get(environment);
    return !summary || summary.active === 0;
  }) ?? null;

  return environmentOrder.map((environment) => {
    const summary = grouped.get(environment);

    if (!summary) {
      return {
        environment,
        status: 'Missing',
        detail: `No ${environment} key is visible yet. Add one before this environment joins the launch path.`,
        recommended: recommendedEnvironment === environment,
      };
    }

    if (summary.active === 0) {
      return {
        environment,
        status: 'Needs replacement',
        detail: `Existing ${environment} keys are inactive. Issue a fresh key before relying on this environment again.`,
        recommended: recommendedEnvironment === environment,
      };
    }

    return {
      environment,
      status: 'Covered',
      detail: `${summary.active} active key(s) currently protect the ${environment} environment boundary.`,
      recommended: false,
    };
  });
}

function buildQuickstartSnippet(createdKey: CreatedGatewayApiKey | null): string | null {
  if (!createdKey) {
    return null;
  }

  return `curl http://127.0.0.1:8080/v1/models \\\n  -H "Authorization: Bearer ${createdKey.plaintext}"`;
}

function buildRotationChecklist(
  keys: GatewayApiKeyRecord[],
  createdKey: CreatedGatewayApiKey | null,
): ApiKeyRotationStep[] {
  const liveKeys = keys.filter((key) => key.environment === 'live' && key.active);

  return [
    {
      id: 'copy-secret',
      title: createdKey ? 'Copy and store the plaintext secret now' : 'Copy plaintext secrets immediately after issuance',
      detail: createdKey
        ? `The newest ${createdKey.environment} secret is visible only once on this screen. Move it into your secret manager before navigating away.`
        : 'The portal never returns plaintext secrets in list APIs, so each new key should be captured at creation time.',
    },
    {
      id: 'verify-environment',
      title: 'Verify the target environment before broad rollout',
      detail: 'Send a small authenticated call from test or staging before switching production workloads to a new credential.',
    },
    {
      id: 'separate-boundaries',
      title: 'Keep environments isolated',
      detail: liveKeys.length > 1
        ? 'Production already has multiple active keys visible, so document ownership before rotating or replacing one.'
        : 'Use separate keys for test, staging, and live so an incident or leak in one environment does not widen the blast radius.',
    },
    {
      id: 'retire-old-secret',
      title: 'Retire the prior secret after cutover',
      detail: 'Once clients confirm the new key works, remove references to the older credential from deployment pipelines and runbooks.',
    },
  ];
}

function buildGuardrails(keys: GatewayApiKeyRecord[]): ApiKeyGuardrail[] {
  const liveSummary = summarizeKeysByEnvironment(keys).find((summary) => summary.environment === 'live');
  const missingEnvironments = buildEnvironmentStrategy(keys)
    .filter((item) => item.status !== 'Covered')
    .map((item) => item.environment);
  const keysMissingExpiry = keys.filter((key) => !key.expires_at_ms);

  const guardrails: ApiKeyGuardrail[] = [];

  if (!liveSummary || liveSummary.active === 0) {
    guardrails.push({
      id: 'live-boundary',
      title: 'Protect production with its own credential boundary',
      detail: 'Do not reuse staging or test credentials in live traffic. Production needs an isolated key with clear ownership.',
      tone: 'warning',
    });
  } else {
    guardrails.push({
      id: 'environment-isolation',
      title: 'Keep each environment on a separate secret',
      detail: 'Independent credentials make it possible to rotate, audit, and revoke access without collateral impact across environments.',
      tone: 'positive',
    });
  }

  if (missingEnvironments.length) {
    guardrails.push({
      id: 'coverage-gap',
      title: 'Close environment coverage before launch expands',
      detail: `The current posture still needs ${missingEnvironments.join(', ')} coverage before the full promotion path is protected.`,
      tone: 'accent',
    });
  }

  if (keysMissingExpiry.length) {
    guardrails.push({
      id: 'expiry-hygiene',
      title: 'Move long-lived keys onto explicit expiry windows',
      detail: `${keysMissingExpiry.length} key(s) currently have no expiry. Add bounded lifetimes so forgotten credentials do not become silent long-tail risk.`,
      tone: 'warning',
    });
  }

  guardrails.push({
    id: 'plaintext-once',
    title: 'Treat plaintext display as a one-time event',
    detail: 'The portal intentionally avoids replaying plaintext keys. Copy once, store safely, and never depend on the UI as secret storage.',
    tone: 'warning',
  });

  return guardrails;
}

export function buildPortalApiKeysViewModel(
  keys: GatewayApiKeyRecord[],
  createdKey: CreatedGatewayApiKey | null,
): PortalApiKeysPageViewModel {
  return {
    keys,
    environment_summaries: summarizeKeysByEnvironment(keys),
    environment_strategy: buildEnvironmentStrategy(keys),
    rotation_checklist: buildRotationChecklist(keys, createdKey),
    guardrails: buildGuardrails(keys),
    created_key: createdKey,
    quickstart_snippet: buildQuickstartSnippet(createdKey),
  };
}
