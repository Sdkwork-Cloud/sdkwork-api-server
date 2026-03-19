import { useEffect, useMemo, useState } from 'react';
import {
  DataTable,
  EmptyState,
  formatCurrency,
  formatUnits,
  InlineButton,
  Surface,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type { LedgerEntry, ProjectBillingSummary } from 'sdkwork-router-portal-types';

import { AccountBalanceFacts } from '../components';
import { getPortalBillingSummary, listPortalBillingLedger } from '../repository';
import { buildPortalAccountViewModel } from '../services';
import type { PortalAccountPageProps } from '../types';

export function PortalAccountPage({ workspace, onNavigate }: PortalAccountPageProps) {
  const [summary, setSummary] = useState<ProjectBillingSummary | null>(null);
  const [ledger, setLedger] = useState<LedgerEntry[]>([]);
  const [status, setStatus] = useState('Loading the financial account posture...');

  useEffect(() => {
    let cancelled = false;

    void Promise.all([getPortalBillingSummary(), listPortalBillingLedger()])
      .then(([nextSummary, nextLedger]) => {
        if (cancelled) {
          return;
        }

        setSummary(nextSummary);
        setLedger(nextLedger);
        setStatus('Financial account posture is synced with the latest billing summary and ledger evidence.');
      })
      .catch((error) => {
        if (!cancelled) {
          setStatus(portalErrorMessage(error));
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  const viewModel = useMemo(() => {
    if (!summary) {
      return null;
    }
    return buildPortalAccountViewModel(summary, ledger);
  }, [ledger, summary]);

  if (!viewModel || !summary) {
    return (
      <Surface detail={status} title="Financial account">
        <EmptyState
          detail="Financial account posture will appear after the portal loads billing summary and ledger evidence."
          title="Preparing account"
        />
      </Surface>
    );
  }

  return (
    <>
      <Tabs className="grid gap-6" defaultValue="balance-summary">
        <TabsList className="w-full justify-start overflow-x-auto">
          <TabsTrigger value="balance-summary">Balance summary</TabsTrigger>
          <TabsTrigger value="ledger-table">Ledger table</TabsTrigger>
          <TabsTrigger value="controls">Controls</TabsTrigger>
        </TabsList>

        <TabsContent className="space-y-6" value="balance-summary">
          <Surface
            actions={
              <div className="flex flex-wrap gap-2">
                <InlineButton onClick={() => onNavigate('credits')} tone="primary">
                  Open credits
                </InlineButton>
                <InlineButton onClick={() => onNavigate('billing')} tone="secondary">
                  Review billing
                </InlineButton>
              </div>
            }
            detail={status}
            title="Cash balance"
          >
            <div className="grid gap-6">
              <div className="portalx-summary-grid">
                {viewModel.cash_balance_cards.map((item) => (
                  <article className="portalx-summary-card" key={item.id}>
                    <span>{item.label}</span>
                    <strong>{item.value}</strong>
                    <p>{item.detail}</p>
                  </article>
                ))}
              </div>

              <div className="grid gap-4 xl:grid-cols-[minmax(0,0.95fr)_minmax(0,1.05fr)]">
                <article className="portal-shell-info-card">
                  <strong className="portal-shell-info-title">Financial account</strong>
                  <p className="portal-shell-info-copy mt-2 text-sm">
                    Project runway, tenant ownership, and booked spend stay visible without a
                    decorative page-top status strip.
                  </p>
                  <div className="mt-4">
                    <AccountBalanceFacts summary={summary} workspace={workspace} />
                  </div>
                </article>

                <article className="portal-shell-info-card">
                  <strong className="portal-shell-info-title">Workspace handoff</strong>
                  <p className="portal-shell-info-copy mt-2 text-sm">
                    Move directly into credits or billing once the current cash posture is clear.
                  </p>
                  <div className="mt-4 flex flex-wrap gap-2">
                    <InlineButton onClick={() => onNavigate('credits')} tone="primary">
                      Open credits
                    </InlineButton>
                    <InlineButton onClick={() => onNavigate('billing')} tone="secondary">
                      Review billing
                    </InlineButton>
                    <InlineButton onClick={() => onNavigate('usage')} tone="ghost">
                      Open usage
                    </InlineButton>
                  </div>
                </article>
              </div>
            </div>
          </Surface>

          <Surface
            detail="Runway, recharge, and ownership boundaries should stay explicit on every financial review."
            title="Operating guardrails"
          >
            <div className="portalx-guardrail-list">
              {viewModel.guardrails.map((item) => (
                <article className="portalx-guardrail-card" key={item.id}>
                  <strong>{item.title}</strong>
                  <p>{item.detail}</p>
                </article>
              ))}
            </div>
          </Surface>
        </TabsContent>

        <TabsContent className="space-y-6" value="ledger-table">
          <Surface detail="The account view should expose the raw ledger table before any higher-level interpretation." title="Ledger table">
            {ledger.length ? (
              <DataTable
                columns={[
                  { key: 'project', label: 'Project', render: (row) => row.project_id },
                  { key: 'units', label: 'Units', render: (row) => formatUnits(row.units) },
                  { key: 'amount', label: 'Amount', render: (row) => formatCurrency(row.amount) },
                ]}
                empty="No ledger entries recorded yet."
                getKey={(row, index) => `${row.project_id}-${row.units}-${index}`}
                rows={ledger}
              />
            ) : (
              <EmptyState
                detail="Ledger lines will appear here as quota and billing activity accumulates."
                title="No ledger entries yet"
              />
            )}
          </Surface>

          <Surface
            detail="Ledger evidence should explain why the financial account looks the way it does right now."
            title="Ledger evidence"
          >
            <div className="portalx-guardrail-list">
              {viewModel.ledger_evidence.map((item) => (
                <article className="portalx-guardrail-card" key={item.id}>
                  <strong>{item.title}</strong>
                  <p>{item.detail}</p>
                </article>
              ))}
            </div>
          </Surface>
        </TabsContent>

        <TabsContent className="space-y-6" value="controls">
          <Surface
            detail="The account module should always direct the user back into the operational loop with a specific next move."
            title="Recommended next financial move"
          >
            <div className="portalx-checklist-grid">
              <article className="portalx-checklist-card">
                <strong>Protect runway before the next launch window</strong>
                <p>Use Credits when you want to add headroom or review coupon-driven top-up options.</p>
                <InlineButton onClick={() => onNavigate('credits')} tone="primary">
                  Open credits
                </InlineButton>
              </article>
              <article className="portalx-checklist-card">
                <strong>Review billing for plan changes</strong>
                <p>Billing remains the lane for bundle selection, subscription shaping, and recovery planning.</p>
                <InlineButton onClick={() => onNavigate('billing')} tone="secondary">
                  Review billing
                </InlineButton>
              </article>
              <article className="portalx-checklist-card">
                <strong>Reconnect money posture with routing and traffic</strong>
                <p>Use Routing and Usage together when commercial posture should inform the next provider or rollout decision.</p>
                <div className="portalx-form-actions">
                  <InlineButton onClick={() => onNavigate('routing')} tone="ghost">
                    Open routing
                  </InlineButton>
                  <InlineButton onClick={() => onNavigate('usage')} tone="ghost">
                    Open usage
                  </InlineButton>
                </div>
              </article>
            </div>
          </Surface>
        </TabsContent>
      </Tabs>
    </>
  );
}
