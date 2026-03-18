import { useEffect, useMemo, useState } from 'react';
import { listRechargePacks, listSubscriptionPlans } from 'sdkwork-router-portal-commerce';
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  formatCurrency,
  formatUnits,
  InlineButton,
  MetricCard,
  Pill,
  Surface,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type {
  ProjectBillingSummary,
  RechargePack,
  SubscriptionPlan,
  UsageRecord,
} from 'sdkwork-router-portal-types';

import { BillingRecommendationCard } from '../components';
import { loadBillingPageData } from '../repository';
import { isRecommendedPack, isRecommendedPlan, recommendBillingChange } from '../services';
import type { PortalBillingPageProps } from '../types';

const emptySummary: ProjectBillingSummary = {
  project_id: '',
  entry_count: 0,
  used_units: 0,
  booked_amount: 0,
  exhausted: false,
};

export function PortalBillingPage({ onNavigate }: PortalBillingPageProps) {
  const [summary, setSummary] = useState<ProjectBillingSummary>(emptySummary);
  const [usageRecords, setUsageRecords] = useState<UsageRecord[]>([]);
  const [status, setStatus] = useState('Loading billing posture...');
  const [checkoutStatus, setCheckoutStatus] = useState(
    'Choose a plan or recharge path to model the next commerce step for this workspace.',
  );
  const [checkoutOpen, setCheckoutOpen] = useState(false);
  const [selectedPlan, setSelectedPlan] = useState<SubscriptionPlan | null>(null);
  const [selectedPack, setSelectedPack] = useState<RechargePack | null>(null);

  useEffect(() => {
    let cancelled = false;

    void loadBillingPageData()
      .then((data) => {
        if (cancelled) {
          return;
        }

        setSummary(data.summary);
        setUsageRecords(data.usage_records);
        setStatus('Live quota posture is paired with a seeded commerce catalog and recent usage demand.');
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

  const recommendation = useMemo(
    () => recommendBillingChange(summary, usageRecords),
    [summary, usageRecords],
  );

  const remainingUnitsLabel =
    summary.remaining_units === null || summary.remaining_units === undefined
      ? 'Unlimited'
      : formatUnits(summary.remaining_units);

  function openPlanPreview(plan: SubscriptionPlan) {
    setSelectedPlan(plan);
    setSelectedPack(null);
    setCheckoutStatus(`${plan.name} selected for checkout preview.`);
    setCheckoutOpen(true);
  }

  function openPackPreview(pack: RechargePack) {
    setSelectedPack(pack);
    setSelectedPlan(null);
    setCheckoutStatus(`${pack.label} selected for recharge preview.`);
    setCheckoutOpen(true);
  }

  const previewTitle = selectedPlan?.name ?? selectedPack?.label ?? recommendation.bundle.title;
  const previewDetail = selectedPlan
    ? `${selectedPlan.price_label} · ${formatUnits(selectedPlan.included_units)} included units`
    : selectedPack
      ? `${selectedPack.price_label} · ${formatUnits(selectedPack.points)} points`
      : recommendation.detail;

  return (
    <>
      <Dialog open={checkoutOpen} onOpenChange={setCheckoutOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Checkout preview</DialogTitle>
            <DialogDescription>{checkoutStatus}</DialogDescription>
          </DialogHeader>

          <div className="grid gap-4 md:grid-cols-2">
            <div className="portal-shell-info-card">
              <p className="text-xs font-semibold uppercase tracking-[0.22em] text-[var(--portal-text-muted)]">Selected offer</p>
              <h3 className="mt-3 text-xl font-semibold text-[var(--portal-text-primary)]">{previewTitle}</h3>
              <p className="portal-shell-info-copy mt-2 text-sm">{previewDetail}</p>
            </div>
            <div className="portal-shell-info-card">
              <p className="text-xs font-semibold uppercase tracking-[0.22em] text-[var(--portal-text-muted)]">Live runway</p>
              <h3 className="mt-3 text-xl font-semibold text-[var(--portal-text-primary)]">{recommendation.runway.label}</h3>
              <p className="portal-shell-info-copy mt-2 text-sm">{recommendation.runway.detail}</p>
            </div>
          </div>

          <DialogFooter>
            <Button onClick={() => setCheckoutOpen(false)} type="button" variant="ghost">
              Close
            </Button>
            <Button onClick={() => onNavigate('credits')} type="button" variant="secondary">
              Open credits
            </Button>
            <Button onClick={() => onNavigate('dashboard')} type="button">
              Return to dashboard
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <div className="portalx-status-row">
        <Pill tone={summary.exhausted ? 'warning' : 'positive'}>
          Live quota: {summary.exhausted ? 'exhausted' : 'healthy'}
        </Pill>
        <Pill tone="seed">Commerce catalog</Pill>
        <span className="portalx-status">{status}</span>
        <InlineButton onClick={() => setCheckoutOpen(true)} tone="primary">
          Checkout preview
        </InlineButton>
        <InlineButton onClick={() => onNavigate('usage')} tone="secondary">
          Open usage
        </InlineButton>
      </div>

      <div className="portalx-metric-grid portalx-metric-grid-dense">
        <MetricCard
          detail="Visible units remaining inside the current project quota."
          label="Remaining units"
          value={remainingUnitsLabel}
        />
        <MetricCard
          detail="Total token units already consumed by this project."
          label="Used units"
          value={formatUnits(summary.used_units)}
        />
        <MetricCard
          detail="Smoothed daily burn estimate derived from recent usage telemetry."
          label="Projected daily burn"
          value={recommendation.runway.daily_units ? `${formatUnits(recommendation.runway.daily_units)} / day` : 'Needs data'}
        />
        <MetricCard
          detail="Booked amount currently visible in the live billing summary."
          label="Booked amount"
          value={formatCurrency(summary.booked_amount)}
        />
      </div>

      <Tabs className="grid gap-6" defaultValue="decision-support">
        <TabsList className="w-full justify-start overflow-x-auto">
          <TabsTrigger value="decision-support">Decision support</TabsTrigger>
          <TabsTrigger value="plan-catalog">Plan catalog</TabsTrigger>
          <TabsTrigger value="recharge-packs">Recharge packs</TabsTrigger>
        </TabsList>

        <TabsContent className="space-y-6" value="decision-support">
          <Surface detail={checkoutStatus} title="Decision support">
            <BillingRecommendationCard recommendation={recommendation} />
          </Surface>

          <div className="grid gap-6 xl:grid-cols-2">
            <Surface detail={recommendation.runway.detail} title="Estimated runway">
              <div className="portalx-readiness-score">
                <span>Projected coverage</span>
                <strong>{recommendation.runway.label}</strong>
              </div>
              <ul className="portalx-bullet-list">
                <li>Observed usage to date: {formatUnits(summary.used_units)} token units.</li>
                <li>Visible remaining quota: {remainingUnitsLabel}.</li>
                <li>
                  {recommendation.runway.daily_units === null
                    ? 'The portal needs more live request history before it can infer a stronger daily burn pace.'
                    : `Estimated burn pace: ${formatUnits(recommendation.runway.daily_units)} token units per day.`}
                </li>
              </ul>
            </Surface>

            <Surface detail={recommendation.bundle.detail} title="Recommended bundle">
              <div className="portalx-bundle-card">
                <Pill tone="positive">{recommendation.plan?.name ?? 'Subscription'}</Pill>
                <strong>{recommendation.bundle.title}</strong>
                <p>{recommendation.detail}</p>
                <ul className="portalx-fact-list">
                  <li>
                    <strong>Subscription</strong>
                    <span>
                      {recommendation.plan
                        ? `${recommendation.plan.name} · ${recommendation.plan.price_label}`
                        : 'No plan recommendation'}
                    </span>
                  </li>
                  <li>
                    <strong>Included units</strong>
                    <span>{recommendation.plan ? formatUnits(recommendation.plan.included_units) : 'Unavailable'}</span>
                  </li>
                  <li>
                    <strong>Recharge buffer</strong>
                    <span>
                      {recommendation.pack
                        ? `${recommendation.pack.label} · ${formatUnits(recommendation.pack.points)}`
                        : 'Optional'}
                    </span>
                  </li>
                </ul>
              </div>
            </Surface>
          </div>

          <Surface
            detail="After choosing a commercial path, the user journey should continue into validation rather than stop at a price table."
            title="Activation path"
          >
            <div className="portalx-checklist-grid">
              <article className="portalx-checklist-card">
                <strong>Return to dashboard and confirm posture</strong>
                <p>Use the command center to verify runway, readiness, and the updated next action after a billing decision.</p>
                <InlineButton onClick={() => onNavigate('dashboard')} tone="primary">
                  Open dashboard
                </InlineButton>
              </article>
              <article className="portalx-checklist-card">
                <strong>Check request demand against the selected plan</strong>
                <p>Go back to Usage when you want to verify that the observed burn pace really matches the selected bundle.</p>
                <InlineButton onClick={() => onNavigate('usage')} tone="secondary">
                  Open usage
                </InlineButton>
              </article>
              <article className="portalx-checklist-card">
                <strong>Keep credentials ready for the next launch window</strong>
                <p>After restoring runway, verify environment keys so billing recovery immediately turns into safe traffic activation.</p>
                <InlineButton onClick={() => onNavigate('api-keys')} tone="ghost">
                  Manage keys
                </InlineButton>
              </article>
            </div>
          </Surface>
        </TabsContent>

        <TabsContent className="space-y-6" value="plan-catalog">
          <Surface detail="Choose the base subscription that matches the projected workspace demand." title="Plan catalog">
            <div className="portalx-plan-grid">
              {listSubscriptionPlans().map((plan) => (
                <article className={`portalx-plan-card ${isRecommendedPlan(plan, recommendation) ? 'portalx-plan-card-featured' : ''}`} key={plan.id}>
                  <p className="portalx-eyebrow">{plan.name}</p>
                  <h3>
                    {plan.price_label}
                    <span>{plan.cadence}</span>
                  </h3>
                  <p>{plan.highlight}</p>
                  <Pill tone={isRecommendedPlan(plan, recommendation) ? 'positive' : 'default'}>
                    {formatUnits(plan.included_units)} included units
                  </Pill>
                  <ul className="portalx-bullet-list">
                    {plan.features.map((feature) => (
                      <li key={feature}>{feature}</li>
                    ))}
                  </ul>
                  <InlineButton onClick={() => openPlanPreview(plan)} tone="primary">
                    {plan.cta}
                  </InlineButton>
                </article>
              ))}
            </div>
          </Surface>
        </TabsContent>

        <TabsContent className="space-y-6" value="recharge-packs">
          <div className="portalx-split-grid portalx-split-grid-wide">
            <Surface detail="Use packs to extend quota without changing the base subscription." title="Recharge packs">
              <div className="portalx-pack-grid">
                {listRechargePacks().map((pack) => (
                  <article className={`portalx-pack-card ${isRecommendedPack(pack, recommendation) ? 'portalx-pack-card-featured' : ''}`} key={pack.id}>
                    <strong>{pack.label}</strong>
                    <span>{formatUnits(pack.points)} points</span>
                    <p>{pack.price_label}</p>
                    <small>{pack.note}</small>
                    <InlineButton onClick={() => openPackPreview(pack)} tone="secondary">
                      Add pack
                    </InlineButton>
                  </article>
                ))}
              </div>
            </Surface>

            <Surface detail="Trust, clarity, and user confidence matter as much as the raw price table." title="Billing confidence">
              <ul className="portalx-bullet-list">
                <li>Quota posture is pulled from the live workspace billing summary.</li>
                <li>Runway guidance is derived from recent usage telemetry rather than a fixed monthly guess.</li>
                <li>Subscription and recharge catalogs stay isolated behind a repository seam for future checkout integration.</li>
                <li>Current recorded usage: {formatUnits(summary.used_units)} token units.</li>
              </ul>
            </Surface>
          </div>
        </TabsContent>
      </Tabs>
    </>
  );
}
