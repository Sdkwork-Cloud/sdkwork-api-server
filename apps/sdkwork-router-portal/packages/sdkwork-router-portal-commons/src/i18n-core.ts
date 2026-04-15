export type PortalLocale = 'en-US' | 'zh-CN';

type TranslationValues = Record<string, string | number>;

let activePortalCoreLocale: PortalLocale = 'en-US';
const runtimePortalMessagesByLocale: Record<PortalLocale, Record<string, string>> = {
  'en-US': {},
  'zh-CN': {},
};
const runtimePortalFallbackByLocale: Record<
  PortalLocale,
  ((text: string) => string | undefined) | undefined
> = {
  'en-US': undefined,
  'zh-CN': undefined,
};
const pendingPortalLocalePreloads = new Map<PortalLocale, Promise<void>>();

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

function resolvePortalTranslation(locale: PortalLocale, text: string): string {
  if (locale === 'en-US') {
    return text;
  }

  return runtimePortalMessagesByLocale[locale][text]
    ?? runtimePortalFallbackByLocale[locale]?.(text)
    ?? text;
}

export async function preloadPortalCoreLocale(locale: PortalLocale): Promise<void> {
  if (locale === 'en-US' || Object.keys(runtimePortalMessagesByLocale[locale]).length > 0) {
    return;
  }

  const existingPending = pendingPortalLocalePreloads.get(locale);
  if (existingPending) {
    await existingPending;
    return;
  }

  const pending = (async () => {
    const { PORTAL_ZH_CN_MESSAGES, translatePortalZhCnFallback } =
      await import('./portalMessages.zh-CN');
    runtimePortalMessagesByLocale[locale] = PORTAL_ZH_CN_MESSAGES;
    runtimePortalFallbackByLocale[locale] = translatePortalZhCnFallback;
  })().finally(() => {
    pendingPortalLocalePreloads.delete(locale);
  });

  pendingPortalLocalePreloads.set(locale, pending);
  await pending;
}

export function translatePortalTextForLocale(
  locale: PortalLocale,
  text: string,
  values?: TranslationValues,
): string {
  const translated = resolvePortalTranslation(locale, text);
  return interpolate(translated, values);
}

export function translatePortalText(text: string, values?: TranslationValues): string {
  return translatePortalTextForLocale(activePortalCoreLocale, text, values);
}
