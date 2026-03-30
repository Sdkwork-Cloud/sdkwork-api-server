export interface SidebarAutoCollapseMetrics {
  viewportWidth: number;
  viewportHeight: number;
  screenWidth?: number;
  screenHeight?: number;
  devicePixelRatio?: number;
}

const COMPACT_VIEWPORT_WIDTH = 1440;
const ROOMY_VIEWPORT_WIDTH = 1600;
const TIGHT_VIEWPORT_HEIGHT = 900;
const HIGH_SCALE_FACTOR = 1.25;
const TIGHT_EFFECTIVE_SCREEN_HEIGHT = 920;

function normalizeMetric(value: number | undefined, fallback: number) {
  if (!Number.isFinite(value) || value === undefined || value <= 0) {
    return fallback;
  }

  return value;
}

export function shouldAutoCollapseSidebar(metrics: SidebarAutoCollapseMetrics) {
  const viewportWidth = normalizeMetric(metrics.viewportWidth, ROOMY_VIEWPORT_WIDTH);
  const viewportHeight = normalizeMetric(metrics.viewportHeight, TIGHT_EFFECTIVE_SCREEN_HEIGHT);
  const devicePixelRatio = Math.max(1, normalizeMetric(metrics.devicePixelRatio, 1));
  const screenWidth = Math.max(viewportWidth, normalizeMetric(metrics.screenWidth, viewportWidth));
  const screenHeight = Math.max(viewportHeight, normalizeMetric(metrics.screenHeight, viewportHeight));
  const effectiveScreenWidth = screenWidth / devicePixelRatio;
  const effectiveScreenHeight = screenHeight / devicePixelRatio;

  if (viewportWidth < COMPACT_VIEWPORT_WIDTH) {
    return true;
  }

  if (viewportWidth >= ROOMY_VIEWPORT_WIDTH) {
    return false;
  }

  const hasTightVerticalSpace =
    viewportHeight < TIGHT_VIEWPORT_HEIGHT || effectiveScreenHeight < TIGHT_EFFECTIVE_SCREEN_HEIGHT;
  const hasDenseDisplayScaling = devicePixelRatio >= HIGH_SCALE_FACTOR;
  const hasLimitedEffectiveScreenWidth = effectiveScreenWidth < ROOMY_VIEWPORT_WIDTH;

  return hasTightVerticalSpace || hasDenseDisplayScaling || hasLimitedEffectiveScreenWidth;
}

export function resolveAutoSidebarCollapsed(
  runtimeWindow: Window | null | undefined = typeof window === 'undefined' ? undefined : window,
) {
  if (!runtimeWindow) {
    return false;
  }

  return shouldAutoCollapseSidebar({
    viewportWidth: runtimeWindow.innerWidth,
    viewportHeight: runtimeWindow.innerHeight,
    screenWidth:
      runtimeWindow.screen?.availWidth ?? runtimeWindow.screen?.width ?? runtimeWindow.innerWidth,
    screenHeight:
      runtimeWindow.screen?.availHeight ??
      runtimeWindow.screen?.height ??
      runtimeWindow.innerHeight,
    devicePixelRatio: runtimeWindow.devicePixelRatio,
  });
}
