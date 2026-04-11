import { startTransition, useEffect, useState } from 'react';

import { usePortalI18n } from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import {
  PortalSiteHero,
  PortalSiteMetricCard,
  PortalSitePanel,
} from 'sdkwork-router-portal-commons/framework/site';

type OpenApiDocument = {
  openapi?: string;
  tags?: Array<{ name?: string }>;
  paths?: Record<string, Record<string, { tags?: string[] } | undefined> | undefined>;
};

type ApiSurfaceDefinition = {
  id: string;
  title: string;
  serverUrl: string;
  primaryAuth: string;
  specEndpoint: string;
  interactiveDocs: string;
  referenceFocus: string;
  commonWorkflow: string;
};

type ApiSurfaceSnapshot = {
  schemaVersion: string;
  operationCount: number;
  tagCount: number;
  routeFamilies: string[];
};

type ApiSurfaceSnapshotState = {
  error?: string;
  loading: boolean;
  snapshot?: ApiSurfaceSnapshot;
};

const apiSurfaces: ApiSurfaceDefinition[] = [
  {
    id: 'gateway',
    title: 'Gateway API',
    serverUrl: '/',
    primaryAuth: 'Gateway API key',
    specEndpoint: '/openapi.json',
    interactiveDocs: '/docs',
    referenceFocus:
      'OpenAI-compatible execution surface for models, responses, embeddings, audio, images, multimodal workloads, and market, coupon, and commercial account workflows.',
    commonWorkflow:
      'List models or issue OpenAI-compatible inference traffic with one gateway API key.',
  },
  {
    id: 'portal',
    title: 'Portal API',
    serverUrl: '/api/portal',
    primaryAuth: 'Portal JWT',
    specEndpoint: '/api/portal/openapi.json',
    interactiveDocs: '/api/portal/docs',
    referenceFocus:
      'Workspace auth, API keys, billing, usage, routing, and commerce endpoints for developers and end users.',
    commonWorkflow:
      'Register or sign in, inspect workspace state, issue an API key, and connect self-service automation.',
  },
];

const HTTP_METHODS = new Set(['get', 'post', 'put', 'patch', 'delete', 'head', 'options', 'trace']);

function openTarget(href: string) {
  if (typeof window === 'undefined') {
    return;
  }

  window.open(href, '_blank', 'noopener,noreferrer');
}

function titleCaseLabel(value: string) {
  return value
    .replace(/[_-]+/g, ' ')
    .replace(/\s+/g, ' ')
    .trim()
    .split(' ')
    .filter(Boolean)
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(' ');
}

function mapTagToLabel(surfaceId: string, rawTag: string) {
  const normalized = rawTag.trim().toLowerCase();
  if (!normalized) {
    return null;
  }

  if (surfaceId === 'gateway') {
    switch (normalized) {
      case 'models':
        return 'Models';
      case 'chat':
        return 'Chat';
      case 'completions':
        return 'Completions';
      case 'responses':
        return 'Responses';
      case 'conversations':
        return 'Conversations';
      case 'embeddings':
        return 'Embeddings';
      case 'moderations':
        return 'Moderations';
      case 'images':
        return 'Images';
      case 'audio':
        return 'Audio';
      case 'files':
        return 'Files';
      case 'uploads':
        return 'Uploads';
      case 'batches':
        return 'Batches';
      case 'vector-stores':
        return 'Vector Stores';
      case 'assistants':
        return 'Assistants';
      case 'threads':
        return 'Threads';
      case 'runs':
        return 'Runs';
      case 'realtime':
        return 'Realtime';
      case 'market':
        return 'Market';
      case 'marketing':
        return 'Marketing';
      case 'commercial':
        return 'Commercial';
      case 'compatibility':
        return 'Compatibility';
      case 'system':
        return 'System';
      default:
        return titleCaseLabel(normalized);
    }
  }

  switch (normalized) {
    case 'auth':
      return 'Authentication';
    case 'workspace':
      return 'Workspace';
    case 'api-keys':
      return 'API Keys';
    case 'billing':
      return 'Billing';
    case 'usage':
      return 'Usage';
    case 'routing':
      return 'Routing';
    case 'commerce':
      return 'Commerce';
    case 'system':
      return 'System';
    default:
      return titleCaseLabel(normalized);
  }
}

function collectRouteFamilies(surfaceId: string, document: OpenApiDocument) {
  const routeFamilies: string[] = [];
  const seen = new Set<string>();

  const append = (rawTag?: string) => {
    if (!rawTag) {
      return;
    }

    const label = mapTagToLabel(surfaceId, rawTag);
    if (!label || seen.has(label)) {
      return;
    }

    seen.add(label);
    routeFamilies.push(label);
  };

  for (const tag of document.tags ?? []) {
    append(tag.name);
  }

  for (const pathItem of Object.values(document.paths ?? {})) {
    if (!pathItem) {
      continue;
    }

    for (const [method, operation] of Object.entries(pathItem)) {
      if (!HTTP_METHODS.has(method.toLowerCase()) || !operation) {
        continue;
      }

      for (const tag of operation.tags ?? []) {
        append(tag);
      }
    }
  }

  return routeFamilies.filter((label) => label !== 'System');
}

function countOperations(document: OpenApiDocument) {
  let operationCount = 0;

  for (const pathItem of Object.values(document.paths ?? {})) {
    if (!pathItem) {
      continue;
    }

    for (const [method, operation] of Object.entries(pathItem)) {
      if (!HTTP_METHODS.has(method.toLowerCase()) || !operation) {
        continue;
      }

      operationCount += 1;
    }
  }

  return operationCount;
}

function summarizeSpec(surfaceId: string, document: OpenApiDocument): ApiSurfaceSnapshot {
  const routeFamilies = collectRouteFamilies(surfaceId, document);

  return {
    schemaVersion: document.openapi ?? 'Unknown',
    operationCount: countOperations(document),
    tagCount: routeFamilies.length,
    routeFamilies,
  };
}

function buildInitialSnapshotState() {
  return Object.fromEntries(
    apiSurfaces.map((surface) => [surface.id, { loading: true }]),
  ) as Record<string, ApiSurfaceSnapshotState>;
}

function formatNumericMetric(value?: number) {
  if (typeof value !== 'number') {
    return '...';
  }

  return String(value);
}

export function PortalApiReferencePage() {
  const { t } = usePortalI18n();
  const [surfaceSnapshots, setSurfaceSnapshots] = useState<Record<string, ApiSurfaceSnapshotState>>(
    () => buildInitialSnapshotState(),
  );

  useEffect(() => {
    let cancelled = false;

    async function loadSnapshots() {
      const results = await Promise.all(
        apiSurfaces.map(async (surface) => {
          try {
            const response = await fetch(surface.specEndpoint, {
              headers: {
                Accept: 'application/json',
              },
            });

            if (!response.ok) {
              throw new Error(`HTTP ${response.status}`);
            }

            const document = (await response.json()) as OpenApiDocument;

            return [
              surface.id,
              {
                loading: false,
                snapshot: summarizeSpec(surface.id, document),
              } satisfies ApiSurfaceSnapshotState,
            ] as const;
          } catch (error) {
            return [
              surface.id,
              {
                error: error instanceof Error ? error.message : 'Unknown error',
                loading: false,
              } satisfies ApiSurfaceSnapshotState,
            ] as const;
          }
        }),
      );

      if (cancelled) {
        return;
      }

      startTransition(() => {
        setSurfaceSnapshots(
          Object.fromEntries(results) as Record<string, ApiSurfaceSnapshotState>,
        );
      });
    }

    void loadSnapshots();

    return () => {
      cancelled = true;
    };
  }, []);

  const gatewaySurface = apiSurfaces[0];
  const portalSurface = apiSurfaces[1];
  const totalOperations = apiSurfaces.reduce(
    (sum, surface) => sum + (surfaceSnapshots[surface.id]?.snapshot?.operationCount ?? 0),
    0,
  );
  const totalTagCount = apiSurfaces.reduce(
    (sum, surface) => sum + (surfaceSnapshots[surface.id]?.snapshot?.tagCount ?? 0),
    0,
  );
  const connectedSurfaceCount = apiSurfaces.filter(
    (surface) => Boolean(surfaceSnapshots[surface.id]?.snapshot),
  ).length;
  const allSurfaceSnapshotsReady = apiSurfaces.every(
    (surface) => surfaceSnapshots[surface.id] && !surfaceSnapshots[surface.id].loading,
  );

  const referenceHighlights = [
    {
      label: 'Live OpenAPI documents',
      value: `${connectedSurfaceCount}/${apiSurfaces.length}`,
      description:
        'Generated directly from the current Rust router implementation so developer docs stay aligned with shipped API behavior.',
    },
    {
      label: 'Operations indexed',
      value: allSurfaceSnapshotsReady ? formatNumericMetric(totalOperations) : '...',
      description:
        'Total operations across gateway and portal developer surfaces, derived from the current live schemas.',
    },
    {
      label: 'Tagged route groups',
      value: allSurfaceSnapshotsReady ? formatNumericMetric(totalTagCount) : '...',
      description:
        'Combined tagged route groups published by the live developer-facing OpenAPI documents.',
    },
    {
      label: 'Authentication boundaries',
      value: '2',
      description:
        'Keep gateway API keys and portal JWT flows visible side by side before integration begins.',
    },
  ] satisfies Array<{
    label: string;
    value: string;
    description: string;
  }>;

  return (
    <div className="space-y-6" data-slot="portal-api-reference-page">
      <PortalSiteHero
        actions={(
          <>
            <Button type="button" onClick={() => openTarget(gatewaySurface.interactiveDocs)}>
              {t('Open gateway docs')}
            </Button>
            <Button
              type="button"
              onClick={() => openTarget(portalSurface.interactiveDocs)}
              variant="secondary"
            >
              {t('Open portal docs')}
            </Button>
            <Button
              type="button"
              onClick={() => openTarget(portalSurface.specEndpoint)}
              variant="ghost"
            >
              {t('Open raw spec')}
            </Button>
          </>
        )}
        aside={(
          <PortalSitePanel
            className="rounded-[28px] border-zinc-200/80 bg-zinc-50/80 dark:border-zinc-800 dark:bg-zinc-900/60"
            description={t('Register or sign in, inspect workspace state, issue an API key, and connect self-service automation.')}
            title={t('Common workflow')}
          >
            {apiSurfaces.map((surface, index) => {
              const status = surfaceSnapshots[surface.id];

              return (
                <div
                  key={surface.id}
                  className="rounded-2xl border border-zinc-200 bg-white px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950"
                >
                  <div className="flex items-center justify-between gap-3">
                    <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                      {t('First request')} {index + 1}
                    </div>
                    <div className="rounded-full border border-zinc-200 px-2.5 py-1 text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:border-zinc-800 dark:text-zinc-300">
                      {status?.loading
                        ? t('Loading live schema')
                        : status?.error
                          ? t('Unavailable')
                          : t('Connected')}
                    </div>
                  </div>
                  <div className="mt-2 text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                    {t(surface.title)}
                  </div>
                  <div className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                    {t(surface.commonWorkflow)}
                  </div>
                </div>
              );
            })}
          </PortalSitePanel>
        )}
        description={t('One developer-facing center for public gateway execution and self-service portal APIs.')}
        eyebrow={t('OpenAPI 3.1')}
        title={t('Explore real-time generated OpenAPI specifications, auth boundaries, and production-ready request flows for gateway and portal integrations.')}
      />

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4" data-slot="portal-api-reference-metrics">
        {referenceHighlights.map((item) => (
          <PortalSiteMetricCard
            key={item.label}
            description={t(item.description)}
            label={t(item.label)}
            value={item.value}
          />
        ))}
      </section>

      <div className="grid gap-4 xl:grid-cols-[minmax(0,1.08fr)_minmax(0,0.92fr)]">
        <PortalSitePanel
          description={t('One developer-facing center for public gateway execution and self-service portal APIs.')}
          title={t('Live OpenAPI documents')}
        >
          <div className="grid gap-4">
            {apiSurfaces.map((surface) => {
              const status = surfaceSnapshots[surface.id];
              const snapshot = status?.snapshot;
              const routeFamilies = snapshot?.routeFamilies ?? [];
              const statusLabel = status?.loading
                ? t('Loading live schema')
                : status?.error
                  ? t('Unavailable')
                  : t('Connected');

              return (
                <div
                  key={surface.id}
                  className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60"
                >
                  <div className="flex flex-wrap items-start justify-between gap-4">
                    <div>
                      <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                        {t('Reference focus')}
                      </div>
                      <div className="mt-2 text-xl font-semibold text-zinc-950 dark:text-zinc-50">
                        {t(surface.title)}
                      </div>
                      <div className="mt-2 max-w-3xl text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                        {t(surface.referenceFocus)}
                      </div>
                    </div>

                    <div className="flex flex-wrap gap-2">
                      <Button type="button" onClick={() => openTarget(surface.interactiveDocs)}>
                        {surface.id === 'gateway' ? t('Open gateway docs') : t('Open portal docs')}
                      </Button>
                      <Button
                        type="button"
                        onClick={() => openTarget(surface.specEndpoint)}
                        variant="secondary"
                      >
                        {t('Open raw spec')}
                      </Button>
                    </div>
                  </div>

                  <div className="mt-5 grid gap-4 md:grid-cols-2 xl:grid-cols-4">
                    <div className="rounded-2xl border border-zinc-200 bg-white px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                      <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                        {t('Server URL')}
                      </div>
                      <div className="mt-2 font-mono text-sm text-zinc-900 dark:text-zinc-100">
                        {surface.serverUrl}
                      </div>
                    </div>

                    <div className="rounded-2xl border border-zinc-200 bg-white px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                      <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                        {t('Primary auth')}
                      </div>
                      <div className="mt-2 text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                        {t(surface.primaryAuth)}
                      </div>
                    </div>

                    <div className="rounded-2xl border border-zinc-200 bg-white px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                      <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                        {t('Spec endpoint')}
                      </div>
                      <div className="mt-2 font-mono text-sm text-zinc-900 dark:text-zinc-100">
                        {surface.specEndpoint}
                      </div>
                    </div>

                    <div className="rounded-2xl border border-zinc-200 bg-white px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                      <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                        {t('Interactive docs')}
                      </div>
                      <div className="mt-2 font-mono text-sm text-zinc-900 dark:text-zinc-100">
                        {surface.interactiveDocs}
                      </div>
                    </div>
                  </div>

                  <div className="mt-4 grid gap-4 md:grid-cols-2 xl:grid-cols-4">
                    <div className="rounded-2xl border border-zinc-200 bg-white px-4 py-4 dark:border-zinc-800 dark:bg-zinc-950">
                      <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                        {t('Schema version')}
                      </div>
                      <div className="mt-2 text-base font-semibold text-zinc-950 dark:text-zinc-50">
                        {snapshot?.schemaVersion ?? '...'}
                      </div>
                    </div>

                    <div className="rounded-2xl border border-zinc-200 bg-white px-4 py-4 dark:border-zinc-800 dark:bg-zinc-950">
                      <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                        {t('Operations indexed')}
                      </div>
                      <div className="mt-2 text-base font-semibold text-zinc-950 dark:text-zinc-50">
                        {formatNumericMetric(snapshot?.operationCount)}
                      </div>
                    </div>

                    <div className="rounded-2xl border border-zinc-200 bg-white px-4 py-4 dark:border-zinc-800 dark:bg-zinc-950">
                      <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                        {t('Tagged route groups')}
                      </div>
                      <div className="mt-2 text-base font-semibold text-zinc-950 dark:text-zinc-50">
                        {formatNumericMetric(snapshot?.tagCount)}
                      </div>
                    </div>

                    <div className="rounded-2xl border border-zinc-200 bg-white px-4 py-4 dark:border-zinc-800 dark:bg-zinc-950">
                      <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                        {t('Live status')}
                      </div>
                      <div className="mt-2 text-base font-semibold text-zinc-950 dark:text-zinc-50">
                        {statusLabel}
                      </div>
                      {status?.error ? (
                        <div className="mt-2 text-xs leading-5 text-zinc-500 dark:text-zinc-400">
                          {t('Live schema unavailable for this surface right now.')}
                        </div>
                      ) : null}
                    </div>
                  </div>

                  <div className="mt-5 grid gap-4 md:grid-cols-[minmax(0,1.06fr)_minmax(0,0.94fr)]">
                    <div className="rounded-2xl border border-zinc-200 bg-white px-4 py-4 dark:border-zinc-800 dark:bg-zinc-950">
                      <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                        {t('Route families')}
                      </div>
                      {routeFamilies.length > 0 ? (
                        <div className="mt-3 flex flex-wrap gap-2">
                          {routeFamilies.map((family) => (
                            <span
                              key={`${surface.id}-${family}`}
                              className="inline-flex items-center rounded-full border border-zinc-200 bg-zinc-50 px-3 py-1 text-xs font-semibold uppercase tracking-[0.14em] text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                            >
                              {t(family)}
                            </span>
                          ))}
                        </div>
                      ) : (
                        <div className="mt-3 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                          {status?.loading
                            ? t('Loading route groups from the live schema.')
                            : t('No tagged route groups are available from the current schema.')}
                        </div>
                      )}
                    </div>

                    <div className="rounded-2xl border border-zinc-200 bg-white px-4 py-4 dark:border-zinc-800 dark:bg-zinc-950">
                      <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                        {t('Common workflow')}
                      </div>
                      <div className="mt-3 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                        {t(surface.commonWorkflow)}
                      </div>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </PortalSitePanel>

        <PortalSitePanel
          description={t('Keep gateway API keys and portal JWT flows visible side by side before integration begins.')}
          title={t('Authentication boundaries')}
        >
          <div className="grid gap-4">
            {apiSurfaces.map((surface) => {
              const status = surfaceSnapshots[surface.id];
              const snapshot = status?.snapshot;

              return (
                <div
                  key={`auth-${surface.id}`}
                  className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60"
                >
                  <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                    {t(surface.title)}
                  </div>
                  <div className="mt-2 text-base font-semibold text-zinc-950 dark:text-zinc-50">
                    {surface.id === 'gateway' ? t('Gateway compatibility') : t('Portal self-service')}
                  </div>
                  <div className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                    {t(surface.referenceFocus)}
                  </div>
                  <div className="mt-4 grid gap-3 sm:grid-cols-2">
                    <div className="rounded-2xl border border-zinc-200 bg-white px-4 py-3 text-sm dark:border-zinc-800 dark:bg-zinc-950">
                      <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                        {t('Primary auth')}
                      </div>
                      <div className="mt-2 font-semibold text-zinc-950 dark:text-zinc-50">
                        {t(surface.primaryAuth)}
                      </div>
                    </div>
                    <div className="rounded-2xl border border-zinc-200 bg-white px-4 py-3 text-sm dark:border-zinc-800 dark:bg-zinc-950">
                      <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                        {t('Operations indexed')}
                      </div>
                      <div className="mt-2 font-semibold text-zinc-950 dark:text-zinc-50">
                        {formatNumericMetric(snapshot?.operationCount)}
                      </div>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </PortalSitePanel>
      </div>
    </div>
  );
}
