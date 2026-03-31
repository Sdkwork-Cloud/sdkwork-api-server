import { translatePortalText, type PortalLocale } from './i18n-core';

let activePortalFormatLocale: PortalLocale = 'en-US';

export function setActivePortalFormatLocale(locale: PortalLocale): void {
  activePortalFormatLocale = locale;
}

export function formatCurrency(amount: number): string {
  return new Intl.NumberFormat(activePortalFormatLocale, {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(amount);
}

export function formatUnits(units: number): string {
  return new Intl.NumberFormat(activePortalFormatLocale).format(units);
}

export function formatDateTime(timestamp: number): string {
  if (!timestamp) {
    return translatePortalText('Pending');
  }

  return new Intl.DateTimeFormat(activePortalFormatLocale, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(timestamp));
}
