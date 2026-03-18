import {
  formatCurrency,
  formatUnits,
} from 'sdkwork-router-portal-commons';
import type {
  LedgerEntry,
  ProjectBillingSummary,
} from 'sdkwork-router-portal-types';

import type {
  FinancialGuardrailItem,
  FinancialMetricItem,
  PortalAccountViewModel,
} from '../types';

function buildCashBalanceCards(
  summary: ProjectBillingSummary,
  ledger: LedgerEntry[],
): FinancialMetricItem[] {
  return [
    {
      id: 'cash-balance',
      label: 'Cash balance',
      value:
        summary.remaining_units === null || summary.remaining_units === undefined
          ? 'Unlimited'
          : formatUnits(summary.remaining_units),
      detail: 'Visible units remaining inside the current financial account boundary.',
    },
    {
      id: 'booked-amount',
      label: 'Booked amount',
      value: formatCurrency(summary.booked_amount),
      detail: 'Commercial spend already recorded against the current project.',
    },
    {
      id: 'ledger-lines',
      label: 'Ledger lines',
      value: String(ledger.length),
      detail: 'Every ledger line keeps financial account posture anchored to recorded evidence.',
    },
  ];
}

function buildLedgerEvidence(
  summary: ProjectBillingSummary,
  ledger: LedgerEntry[],
): FinancialGuardrailItem[] {
  if (!ledger.length) {
    return [
      {
        id: 'ledger-empty',
        title: 'Ledger evidence is waiting for financial activity',
        detail: 'As credits, usage, and quota events land, the financial account will summarize the cash and unit trail here.',
      },
    ];
  }

  const totalLedgerUnits = ledger.reduce((sum, entry) => sum + entry.units, 0);
  const largestUnitEntry = [...ledger].sort((left, right) => right.units - left.units)[0];
  const largestAmountEntry = [...ledger].sort((left, right) => right.amount - left.amount)[0];

  return [
    {
      id: 'ledger-coverage',
      title: 'Ledger coverage matches the visible spend posture',
      detail: `${ledger.length} ledger line(s) account for ${formatUnits(totalLedgerUnits)} units across ${formatCurrency(summary.booked_amount)} of booked spend.`,
    },
    {
      id: 'largest-unit-entry',
      title: 'Largest visible unit movement',
      detail: `${formatUnits(largestUnitEntry.units)} units sit on the heaviest visible ledger line for project ${largestUnitEntry.project_id}.`,
    },
    {
      id: 'largest-amount-entry',
      title: 'Largest booked amount line',
      detail: `${formatCurrency(largestAmountEntry.amount)} is the highest single visible booked amount in the current ledger scope.`,
    },
  ];
}

function buildGuardrails(summary: ProjectBillingSummary): FinancialGuardrailItem[] {
  return [
    {
      id: 'runway',
      title: summary.exhausted ? 'Financial runway needs immediate action' : 'Financial runway is visible',
      detail: summary.exhausted
        ? 'Quota is exhausted, so billing recovery and credits review must happen before the next traffic window.'
        : summary.remaining_units === null || summary.remaining_units === undefined
          ? 'The current financial account boundary shows unlimited visible runway.'
          : `${formatUnits(summary.remaining_units)} units remain before the visible quota boundary is reached.`,
    },
    {
      id: 'discipline',
      title: 'Financial account stays separate from personal identity',
      detail: 'Use User for profile and password posture, and keep recharge, ledger, and booked-amount review inside Account.',
    },
    {
      id: 'evidence',
      title: 'Ledger evidence should drive money decisions',
      detail: 'Treat top-ups, plan changes, and runway review as evidence-backed actions rather than assumptions.',
    },
  ];
}

export function buildPortalAccountViewModel(
  summary: ProjectBillingSummary,
  ledger: LedgerEntry[],
): PortalAccountViewModel {
  return {
    billing_summary: summary,
    ledger,
    cash_balance_cards: buildCashBalanceCards(summary, ledger),
    ledger_evidence: buildLedgerEvidence(summary, ledger),
    guardrails: buildGuardrails(summary),
  };
}
