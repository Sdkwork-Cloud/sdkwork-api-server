const PLATFORM_ALIASES = new Map([
  ['win32', 'windows'],
  ['windows', 'windows'],
  ['darwin', 'macos'],
  ['macos', 'macos'],
  ['linux', 'linux'],
]);

const ARCH_ALIASES = new Map([
  ['x64', 'x64'],
  ['x86_64', 'x64'],
  ['amd64', 'x64'],
  ['arm64', 'arm64'],
  ['aarch64', 'arm64'],
]);

const TARGET_TRIPLE_SPECS = new Map([
  ['x86_64-pc-windows-msvc', { platform: 'windows', arch: 'x64' }],
  ['aarch64-pc-windows-msvc', { platform: 'windows', arch: 'arm64' }],
  ['x86_64-unknown-linux-gnu', { platform: 'linux', arch: 'x64' }],
  ['aarch64-unknown-linux-gnu', { platform: 'linux', arch: 'arm64' }],
  ['x86_64-apple-darwin', { platform: 'macos', arch: 'x64' }],
  ['aarch64-apple-darwin', { platform: 'macos', arch: 'arm64' }],
]);

export const DESKTOP_TARGET_ENV_VAR = 'SDKWORK_DESKTOP_TARGET';
export const DESKTOP_TARGET_PLATFORM_ENV_VAR = 'SDKWORK_DESKTOP_TARGET_PLATFORM';
export const DESKTOP_TARGET_ARCH_ENV_VAR = 'SDKWORK_DESKTOP_TARGET_ARCH';

export function normalizeDesktopPlatform(platform = process.platform) {
  const normalized = PLATFORM_ALIASES.get(String(platform).trim().toLowerCase());
  if (!normalized) {
    throw new Error(`Unsupported desktop release platform: ${platform}`);
  }

  return normalized;
}

export function normalizeDesktopArch(arch = process.arch) {
  const normalized = ARCH_ALIASES.get(String(arch).trim().toLowerCase());
  if (!normalized) {
    throw new Error(`Unsupported desktop release architecture: ${arch}`);
  }

  return normalized;
}

export function buildDesktopTargetTriple({ platform, arch }) {
  const normalizedPlatform = normalizeDesktopPlatform(platform);
  const normalizedArch = normalizeDesktopArch(arch);

  if (normalizedPlatform === 'windows') {
    return normalizedArch === 'arm64'
      ? 'aarch64-pc-windows-msvc'
      : 'x86_64-pc-windows-msvc';
  }

  if (normalizedPlatform === 'linux') {
    return normalizedArch === 'arm64'
      ? 'aarch64-unknown-linux-gnu'
      : 'x86_64-unknown-linux-gnu';
  }

  return normalizedArch === 'arm64'
    ? 'aarch64-apple-darwin'
    : 'x86_64-apple-darwin';
}

export function parseDesktopTargetTriple(targetTriple) {
  const normalizedTargetTriple = String(targetTriple ?? '').trim();
  const spec = TARGET_TRIPLE_SPECS.get(normalizedTargetTriple);
  if (!spec) {
    throw new Error(`Unsupported desktop release target triple: ${targetTriple}`);
  }

  return {
    ...spec,
    targetTriple: normalizedTargetTriple,
  };
}

export function resolveDesktopReleaseTarget({
  targetTriple,
  platform,
  arch,
  env = process.env,
} = {}) {
  const requestedTargetTriple = firstNonEmpty(
    targetTriple,
    env?.[DESKTOP_TARGET_ENV_VAR],
  );

  if (requestedTargetTriple) {
    return parseDesktopTargetTriple(requestedTargetTriple);
  }

  const resolvedPlatform = normalizeDesktopPlatform(
    firstNonEmpty(platform, env?.[DESKTOP_TARGET_PLATFORM_ENV_VAR], process.platform),
  );
  const resolvedArch = normalizeDesktopArch(
    firstNonEmpty(arch, env?.[DESKTOP_TARGET_ARCH_ENV_VAR], process.arch),
  );

  return {
    platform: resolvedPlatform,
    arch: resolvedArch,
    targetTriple: buildDesktopTargetTriple({
      platform: resolvedPlatform,
      arch: resolvedArch,
    }),
  };
}

export function buildDesktopReleaseEnv({
  env = process.env,
  targetTriple,
  platform,
  arch,
} = {}) {
  const target = resolveDesktopReleaseTarget({
    targetTriple,
    platform,
    arch,
    env,
  });

  return {
    ...env,
    [DESKTOP_TARGET_ENV_VAR]: target.targetTriple,
    [DESKTOP_TARGET_PLATFORM_ENV_VAR]: target.platform,
    [DESKTOP_TARGET_ARCH_ENV_VAR]: target.arch,
  };
}

function firstNonEmpty(...values) {
  for (const value of values) {
    if (typeof value === 'string' && value.trim().length > 0) {
      return value.trim();
    }
  }

  return '';
}
