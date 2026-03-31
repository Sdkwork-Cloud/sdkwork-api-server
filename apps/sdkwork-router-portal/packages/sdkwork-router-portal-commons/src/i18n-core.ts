export type PortalLocale = 'en-US' | 'zh-CN';

type TranslationValues = Record<string, string | number>;

let activePortalCoreLocale: PortalLocale = 'en-US';

const portalCoreMessages: Record<Exclude<PortalLocale, 'en-US'>, Record<string, string>> = {
  'zh-CN': {
    Pending: '待处理',
    'Unknown provider': '未知 Provider',
    unknown: '未知',
    Unavailable: '不可用',
    'Predictable order': '可预测顺序',
    'Traffic distribution': '流量分配',
    'Reliability guardrails': '可靠性护栏',
    'Regional preference': '区域偏好',
    'Platform fallback': '平台回退',
    'Adaptive routing': '自适应路由',
    'Default provider': '默认 Provider',
    'Max cost': '最大成本',
    Open: '开放',
    'Max latency': '最大延迟',
    'Preferred region': '偏好区域',
    Auto: '自动',
    'A default provider acts as the stable fallback when multiple candidates remain eligible.': '当多个候选仍可用时，默认 Provider 会作为稳定回退存在。',
    'Keep a cost ceiling visible so route posture reflects commercial intent, not only technical possibility.': '保持成本上限可见，让路由姿态体现商业意图，而不只体现技术可行性。',
    'Latency guardrails let the workspace make reliability posture explicit before traffic starts flowing.': '延迟护栏让工作区在流量开始前就能明确可靠性姿态。',
    'The active route preview should always show the region signal that influenced provider selection.': '当前路由预览应始终展示影响 Provider 选择的区域信号。',
    '{source} used {strategy}{regionSuffix}.': '{source} 使用了 {strategy}{regionSuffix}。',
    ' in {region}': '，区域 {region}',
  },
};

function interpolate(text: string, values?: TranslationValues): string {
  if (!values) {
    return text;
  }

  return Object.entries(values).reduce(
    (result, [key, value]) => result.replaceAll(`{${key}}`, String(value)),
    text,
  );
}

export function setActivePortalCoreLocale(locale: PortalLocale): void {
  activePortalCoreLocale = locale;
}

export function translatePortalText(text: string, values?: TranslationValues): string {
  const translated = activePortalCoreLocale === 'en-US'
    ? text
    : portalCoreMessages[activePortalCoreLocale][text] ?? text;

  return interpolate(translated, values);
}
