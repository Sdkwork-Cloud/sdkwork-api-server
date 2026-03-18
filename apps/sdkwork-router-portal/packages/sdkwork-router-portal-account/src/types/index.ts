import type {
  LedgerEntry,
  PortalRouteKey,
  PortalWorkspaceSummary,
  ProjectBillingSummary,
} from 'sdkwork-router-portal-types';

export interface PortalAccountPageProps {
  workspace: PortalWorkspaceSummary | null;
  onNavigate: (route: PortalRouteKey) => void;
}

export interface FinancialMetricItem {
  id: string;
  label: string;
  value: string;
  detail: string;
}

export interface FinancialGuardrailItem {
  id: string;
  title: string;
  detail: string;
}

export interface PortalAccountViewModel {
  billing_summary: ProjectBillingSummary;
  ledger: LedgerEntry[];
  cash_balance_cards: FinancialMetricItem[];
  ledger_evidence: FinancialGuardrailItem[];
  guardrails: FinancialGuardrailItem[];
}
