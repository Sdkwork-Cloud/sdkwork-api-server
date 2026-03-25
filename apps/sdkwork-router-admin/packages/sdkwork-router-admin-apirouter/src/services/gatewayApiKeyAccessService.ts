export type ApiKeySetupClientId =
  | 'codex'
  | 'claude-code'
  | 'opencode'
  | 'gemini'
  | 'openclaw';

export type ApiKeySetupCompatibility = 'openai' | 'anthropic' | 'gemini';
export type ApiKeySetupInstallMode = 'standard' | 'env' | 'both';
export type ApiKeySetupEnvScope = 'user' | 'system';
export type ApiKeySetupSnippetLanguage = 'json' | 'toml' | 'bash' | 'text';

export interface ApiKeySetupSnippet {
  id: string;
  title: string;
  target: string;
  language: ApiKeySetupSnippetLanguage;
  content: string;
}

export interface ApiKeySetupInstance {
  id: string;
  label: string;
  detail?: string | null;
}

export interface ApiKeyClientInstallRequest {
  clientId: ApiKeySetupClientId;
  installMode?: ApiKeySetupInstallMode;
  envScope?: ApiKeySetupEnvScope;
  provider: {
    id: string;
    channelId: string;
    name: string;
    baseUrl: string;
    apiKey: string;
    compatibility: ApiKeySetupCompatibility;
    models: Array<{ id: string; name: string }>;
  };
  openClaw?: {
    instanceIds: string[];
  };
}

export interface ApiKeyClientInstallResult {
  clientId: ApiKeySetupClientId;
  writtenFiles: Array<{ path: string; action: 'created' | 'updated' }>;
  updatedEnvironments: Array<{
    scope: ApiKeySetupEnvScope;
    shell: 'powershell' | 'sh';
    target: string;
    variables: string[];
  }>;
  updatedInstanceIds: string[];
}

export interface ApiKeyQuickSetupPlan {
  id: ApiKeySetupClientId;
  label: string;
  description: string;
  compatibility: ApiKeySetupCompatibility;
  snippets: ApiKeySetupSnippet[];
  request: ApiKeyClientInstallRequest;
  applyLabel: string;
  requiresInstances?: boolean;
}

export interface ApiKeyQuickSetupInput {
  hashedKey: string;
  label: string;
  plaintextKey: string;
  gatewayBaseUrl: string;
  defaults?: Partial<{
    openaiModel: string;
    anthropicModel: string;
    geminiModel: string;
  }>;
}

type TauriWindowLike = Window & {
  __TAURI__?: unknown;
  __TAURI_INTERNALS__?: TauriInternalsLike;
  isTauri?: boolean;
};

type TauriInternalsLike = {
  invoke?: <T>(command: string, args?: Record<string, unknown>) => Promise<T>;
};

const DEFAULT_GATEWAY_BASE_URL = 'http://127.0.0.1:8080';

function resolveWindow(): TauriWindowLike | null {
  if (typeof window === 'undefined') {
    return null;
  }

  return window as TauriWindowLike;
}

function trimTrailingSlash(value: string): string {
  return value.replace(/\/+$/g, '');
}

function sanitizeModelId(value: string | undefined, fallback: string): string {
  const normalized = value?.trim();
  return normalized || fallback;
}

function joinUrl(baseUrl: string, path: string): string {
  const normalizedBase = trimTrailingSlash(baseUrl);
  const normalizedPath = path.startsWith('/') ? path : `/${path}`;
  return `${normalizedBase}${normalizedPath}`;
}

function json(value: unknown): string {
  return JSON.stringify(value, null, 2);
}

function buildProviderId(hashedKey: string, clientId: ApiKeySetupClientId): string {
  return `api-key-${clientId}-${hashedKey.slice(0, 12)}`;
}

function isDesktopRuntime(): boolean {
  const currentWindow = resolveWindow();
  return Boolean(
    currentWindow?.isTauri ||
      currentWindow?.__TAURI__ ||
      currentWindow?.__TAURI_INTERNALS__,
  );
}

async function invokeDesktopCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const invoke = resolveWindow()?.__TAURI_INTERNALS__?.invoke;
  if (typeof invoke !== 'function') {
    throw new Error('Tauri invoke bridge is unavailable.');
  }

  return invoke<T>(command, args);
}

export async function resolveGatewayBaseUrl(): Promise<string> {
  if (!isDesktopRuntime()) {
    return DEFAULT_GATEWAY_BASE_URL;
  }

  try {
    const baseUrl = await invokeDesktopCommand<string>('runtime_base_url');
    return baseUrl || DEFAULT_GATEWAY_BASE_URL;
  } catch {
    return DEFAULT_GATEWAY_BASE_URL;
  }
}

export async function listApiKeyInstances(): Promise<ApiKeySetupInstance[]> {
  if (!isDesktopRuntime()) {
    return [];
  }

  try {
    return await invokeDesktopCommand<ApiKeySetupInstance[]>('list_api_key_instances');
  } catch {
    return [];
  }
}

export async function applyApiKeyQuickSetup(
  request: ApiKeyClientInstallRequest,
): Promise<ApiKeyClientInstallResult> {
  return invokeDesktopCommand<ApiKeyClientInstallResult>('install_api_router_client_setup', {
    request,
  });
}

export function buildApiKeyCurlSnippet(
  gatewayBaseUrl: string,
  plaintextKey: string,
  modelId = 'gpt-5.4',
): string {
  return [
    `curl ${joinUrl(gatewayBaseUrl, '/v1/chat/completions')} \\`,
    `  -H "Authorization: Bearer ${plaintextKey}" \\`,
    '  -H "Content-Type: application/json" \\',
    "  -d '{",
    `    "model": "${modelId}",`,
    '    "messages": [',
    '      { "role": "user", "content": "Ping SDKWork Router" }',
    '    ]',
    "  }'",
  ].join('\n');
}

export function buildApiKeyQuickSetupPlans(input: ApiKeyQuickSetupInput): ApiKeyQuickSetupPlan[] {
  const gatewayBaseUrl = trimTrailingSlash(input.gatewayBaseUrl || DEFAULT_GATEWAY_BASE_URL);
  const openaiBaseUrl = joinUrl(gatewayBaseUrl, '/v1');
  const anthropicBaseUrl = joinUrl(gatewayBaseUrl, '/v1');
  const geminiBaseUrl = gatewayBaseUrl;
  const openaiModel = sanitizeModelId(input.defaults?.openaiModel, 'gpt-5.4');
  const anthropicModel = sanitizeModelId(input.defaults?.anthropicModel, 'claude-sonnet-4');
  const geminiModel = sanitizeModelId(input.defaults?.geminiModel, 'gemini-2.5-pro');
  const routerName = 'SDKWork Router';
  const sharedDescription =
    'Use the current Api key directly against the SDKWork Router gateway without introducing a second credential boundary.';

  return [
    {
      id: 'codex',
      label: 'Codex',
      description: `${sharedDescription} Codex stays on the OpenAI-compatible responses stack.`,
      compatibility: 'openai',
      applyLabel: 'Apply setup',
      request: {
        clientId: 'codex',
        provider: {
          id: buildProviderId(input.hashedKey, 'codex'),
          channelId: 'openai',
          name: routerName,
          baseUrl: openaiBaseUrl,
          apiKey: input.plaintextKey,
          compatibility: 'openai',
          models: [{ id: openaiModel, name: openaiModel }],
        },
      },
      snippets: [
        {
          id: 'config',
          title: 'Config',
          target: '~/.codex/config.toml',
          language: 'toml',
          content: [
            `model = "${openaiModel}"`,
            'model_provider = "api_router"',
            '',
            '[model_providers.api_router]',
            `name = "${routerName}"`,
            `base_url = "${openaiBaseUrl}"`,
            'wire_api = "responses"',
            'requires_openai_auth = true',
          ].join('\n'),
        },
        {
          id: 'auth',
          title: 'Auth',
          target: '~/.codex/auth.json',
          language: 'json',
          content: json({
            auth_mode: 'apikey',
            OPENAI_API_KEY: input.plaintextKey,
          }),
        },
      ],
    },
    {
      id: 'claude-code',
      label: 'Claude Code',
      description:
        'Claude Code uses the Anthropic-compatible route exposed by the gateway and keeps the same Api key.',
      compatibility: 'anthropic',
      applyLabel: 'Apply setup',
      request: {
        clientId: 'claude-code',
        provider: {
          id: buildProviderId(input.hashedKey, 'claude-code'),
          channelId: 'anthropic',
          name: routerName,
          baseUrl: anthropicBaseUrl,
          apiKey: input.plaintextKey,
          compatibility: 'anthropic',
          models: [{ id: anthropicModel, name: anthropicModel }],
        },
      },
      snippets: [
        {
          id: 'settings',
          title: 'Settings',
          target: '~/.claude/settings.json',
          language: 'json',
          content: json({
            $schema: 'https://json.schemastore.org/claude-code-settings.json',
            model: anthropicModel,
            env: {
              ANTHROPIC_AUTH_TOKEN: input.plaintextKey,
              ANTHROPIC_BASE_URL: anthropicBaseUrl,
            },
          }),
        },
      ],
    },
    {
      id: 'opencode',
      label: 'OpenCode',
      description:
        'OpenCode uses the OpenAI-compatible provider block and the same routed Api key.',
      compatibility: 'openai',
      applyLabel: 'Apply setup',
      request: {
        clientId: 'opencode',
        provider: {
          id: buildProviderId(input.hashedKey, 'opencode'),
          channelId: 'openai',
          name: routerName,
          baseUrl: openaiBaseUrl,
          apiKey: input.plaintextKey,
          compatibility: 'openai',
          models: [{ id: openaiModel, name: openaiModel }],
        },
      },
      snippets: [
        {
          id: 'config',
          title: 'Config',
          target: '~/.config/opencode/opencode.json',
          language: 'json',
          content: json({
            $schema: 'https://opencode.ai/config.json',
            provider: {
              'api-router': {
                npm: '@ai-sdk/openai',
                name: routerName,
                options: {
                  baseURL: openaiBaseUrl,
                },
                models: {
                  [openaiModel]: {
                    name: `${routerName} / ${openaiModel}`,
                  },
                },
              },
            },
            model: `api-router/${openaiModel}`,
          }),
        },
        {
          id: 'auth',
          title: 'Auth',
          target: '~/.local/share/opencode/auth.json',
          language: 'json',
          content: json({
            'api-router': {
              type: 'api',
              key: input.plaintextKey,
            },
          }),
        },
      ],
    },
    {
      id: 'gemini',
      label: 'Gemini',
      description:
        'Gemini CLI uses the gateway Gemini-compatible compatibility routes while keeping this Api key as the only secret.',
      compatibility: 'gemini',
      applyLabel: 'Apply setup',
      request: {
        clientId: 'gemini',
        provider: {
          id: buildProviderId(input.hashedKey, 'gemini'),
          channelId: 'gemini',
          name: routerName,
          baseUrl: geminiBaseUrl,
          apiKey: input.plaintextKey,
          compatibility: 'gemini',
          models: [{ id: geminiModel, name: geminiModel }],
        },
      },
      snippets: [
        {
          id: 'settings',
          title: 'Settings',
          target: '~/.gemini/settings.json',
          language: 'json',
          content: json({
            model: {
              name: geminiModel,
            },
            security: {
              auth: {
                selectedType: 'gemini-api-key',
              },
            },
          }),
        },
        {
          id: 'env',
          title: 'Environment',
          target: '~/.gemini/.env',
          language: 'bash',
          content: [
            `GEMINI_API_KEY="${input.plaintextKey}"`,
            `GOOGLE_GEMINI_BASE_URL="${geminiBaseUrl}"`,
            'GEMINI_API_KEY_AUTH_MECHANISM="bearer"',
          ].join('\n'),
        },
      ],
    },
    {
      id: 'openclaw',
      label: 'OpenClaw',
      description:
        'OpenClaw writes a provider manifest into the selected local instances and points them at the routed gateway endpoint.',
      compatibility: 'openai',
      applyLabel: 'Apply setup',
      requiresInstances: true,
      request: {
        clientId: 'openclaw',
        provider: {
          id: buildProviderId(input.hashedKey, 'openclaw'),
          channelId: 'openai',
          name: routerName,
          baseUrl: openaiBaseUrl,
          apiKey: input.plaintextKey,
          compatibility: 'openai',
          models: [{ id: openaiModel, name: openaiModel }],
        },
      },
      snippets: [
        {
          id: 'manifest',
          title: 'Provider manifest',
          target: '~/.openclaw/instances/<instance-id>/providers/provider-api-router.json',
          language: 'json',
          content: json({
            provider: 'api-router',
            endpoint: openaiBaseUrl,
            apiKey: input.plaintextKey,
            defaultModelId: openaiModel,
            label: input.label,
          }),
        },
      ],
    },
  ];
}
