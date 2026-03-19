import { useEffect, useMemo, useState } from 'react';
import type { FormEvent } from 'react';
import { listCouponOffers, redeemSeedCoupon } from 'sdkwork-router-portal-commerce';
import {
  Button,
  DataTable,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  EmptyState,
  FormField,
  formatCurrency,
  formatUnits,
  InlineButton,
  Input,
  Pill,
  Surface,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type { CouponOffer, LedgerEntry, ProjectBillingSummary } from 'sdkwork-router-portal-types';

import { CouponImpactCard } from '../components';
import { loadCreditsPageData } from '../repository';
import {
  buildCouponImpactPreview,
  buildRecommendedCouponOffer,
  buildRedemptionGuardrails,
  recommendCouponOffer,
} from '../services';
import type { PortalCreditsPageProps } from '../types';

const emptySummary: ProjectBillingSummary = {
  project_id: '',
  entry_count: 0,
  used_units: 0,
  booked_amount: 0,
  exhausted: false,
};

export function PortalCreditsPage({ onNavigate }: PortalCreditsPageProps) {
  const [summary, setSummary] = useState<ProjectBillingSummary>(emptySummary);
  const [ledger, setLedger] = useState<LedgerEntry[]>([]);
  const [couponCode, setCouponCode] = useState('');
  const [selectedOffer, setSelectedOffer] = useState<CouponOffer | null>(null);
  const [couponStatus, setCouponStatus] = useState('Redeem workspace offers and keep points posture visible before traffic spikes.');
  const [status, setStatus] = useState('Loading points posture...');
  const [redeemDialogOpen, setRedeemDialogOpen] = useState(false);

  useEffect(() => {
    let cancelled = false;

    void loadCreditsPageData()
      .then((data) => {
        if (cancelled) {
          return;
        }

        setSummary(data.summary);
        setLedger(data.ledger);
        const nextOffer = recommendCouponOffer(data.summary);
        setSelectedOffer(nextOffer);
        setCouponCode(nextOffer.code);
        setStatus('Live billing data is mapped into a points-oriented portal view.');
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

  function handleCouponSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const offer = redeemSeedCoupon(couponCode);
    if (!offer) {
      setCouponStatus('Coupon code not recognized in the current seeded commerce catalog.');
      setSelectedOffer(null);
      return;
    }

    setSelectedOffer(offer);
    setCouponStatus(`${offer.code} accepted for preview: ${offer.benefit}. Backend redemption can replace this seam without changing the UI contract.`);
  }

  function previewOffer(offer: CouponOffer) {
    setSelectedOffer(offer);
    setCouponCode(offer.code);
    setCouponStatus(`${offer.code} loaded into the redeem workflow for preview.`);
    setRedeemDialogOpen(true);
  }

  const remainingUnits = summary.remaining_units ?? 0;
  const couponPreview = useMemo(
    () => (selectedOffer ? buildCouponImpactPreview(summary, selectedOffer) : null),
    [selectedOffer, summary],
  );
  const recommendedOffer = useMemo(() => buildRecommendedCouponOffer(summary), [summary]);
  const guardrails = useMemo(() => buildRedemptionGuardrails(summary), [summary]);

  return (
    <>
      <Dialog open={redeemDialogOpen} onOpenChange={setRedeemDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Redeem credits</DialogTitle>
            <DialogDescription>{couponStatus}</DialogDescription>
          </DialogHeader>

          <form className="grid gap-4" onSubmit={handleCouponSubmit}>
            <FormField hint="Seed coupons can be replaced by a live redemption backend without changing the page contract." label="Coupon code">
              <Input
                onChange={(event) => setCouponCode(event.target.value)}
                placeholder="WELCOME100"
                value={couponCode}
              />
            </FormField>
            {couponPreview ? <CouponImpactCard preview={couponPreview} /> : null}
            <DialogFooter>
              <Button onClick={() => setRedeemDialogOpen(false)} type="button" variant="ghost">
                Close
              </Button>
              <Button type="submit">Preview redemption</Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
      <Tabs className="grid gap-6" defaultValue="overview">
        <TabsList className="w-full justify-start overflow-x-auto">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="offer-catalog">Offer catalog</TabsTrigger>
          <TabsTrigger value="ledger">Ledger</TabsTrigger>
        </TabsList>

        <TabsContent className="space-y-6" value="overview">
          <div className="portalx-split-grid portalx-split-grid-wide">
            <Surface
              actions={
                <div className="flex flex-wrap gap-2">
                  <InlineButton onClick={() => setRedeemDialogOpen(true)} tone="primary">
                    Redeem credits
                  </InlineButton>
                  <InlineButton onClick={() => onNavigate('billing')} tone="secondary">
                    Review billing
                  </InlineButton>
                </div>
              }
              detail={status}
              title="Recommended offer"
            >
              <div className="grid gap-6">
                <CouponImpactCard preview={recommendedOffer.preview} />
                <div className="portalx-summary-grid">
                  <article className="portalx-summary-card">
                    <span>Available points</span>
                    <strong>{formatUnits(remainingUnits)}</strong>
                    <p>Remaining token units within the current quota boundary.</p>
                  </article>
                  <article className="portalx-summary-card">
                    <span>Used points</span>
                    <strong>{formatUnits(summary.used_units)}</strong>
                    <p>Consumed token units recorded for this project.</p>
                  </article>
                  <article className="portalx-summary-card">
                    <span>Ledger entries</span>
                    <strong>{formatUnits(summary.entry_count)}</strong>
                    <p>Ledger entries currently visible in the billing read model.</p>
                  </article>
                  <article className="portalx-summary-card">
                    <span>Booked amount</span>
                    <strong>{formatCurrency(summary.booked_amount)}</strong>
                    <p>Booked amount associated with usage to date.</p>
                  </article>
                </div>
                <ul className="portalx-fact-list">
                  <li>
                    <strong>Coupon code</strong>
                    <span>{recommendedOffer.offer.code}</span>
                  </li>
                  <li>
                    <strong>Benefit</strong>
                    <span>{recommendedOffer.offer.benefit}</span>
                  </li>
                  <li>
                    <strong>Best for</strong>
                    <span>{recommendedOffer.offer.description}</span>
                  </li>
                </ul>
              </div>
            </Surface>

            <Surface
              detail="Use coupons as a productized quota extension path, but keep the rules visible so launch motion stays safe."
              title="Redemption guardrails"
            >
              <div className="portalx-guardrail-list">
                {guardrails.map((guardrail) => (
                  <article className="portalx-guardrail-card" key={guardrail.id}>
                    <Pill tone={guardrail.tone}>{guardrail.title}</Pill>
                    <p>{guardrail.detail}</p>
                  </article>
                ))}
              </div>
            </Surface>
          </div>

          {couponPreview ? (
            <Surface detail={couponStatus} title="Redemption impact">
              <CouponImpactCard preview={couponPreview} />
            </Surface>
          ) : null}
        </TabsContent>

        <TabsContent className="space-y-6" value="offer-catalog">
          <Surface detail="Current workspace offers prepared behind a commerce repository seam." title="Offer catalog">
            <ul className="portalx-offer-list">
              {listCouponOffers().map((offer) => (
                <li className="portalx-checklist-card" key={offer.code}>
                  <strong>{offer.title}</strong>
                  <span>{offer.benefit}</span>
                  <p>{offer.description}</p>
                  <InlineButton onClick={() => previewOffer(offer)} tone="secondary">
                    Preview offer
                  </InlineButton>
                </li>
              ))}
            </ul>
          </Surface>

          <Surface
            detail="Credits should flow naturally into the next commercial decision instead of ending at coupon redemption."
            title="Recharge decision"
          >
            <div className="portalx-checklist-grid">
              <article className="portalx-checklist-card">
                <strong>Escalate to billing when coupon support is not enough</strong>
                <p>Use subscriptions and recharge packs when demand has moved beyond one-off promotional top-ups.</p>
                <InlineButton onClick={() => onNavigate('billing')} tone="primary">
                  Review billing
                </InlineButton>
              </article>
              <article className="portalx-checklist-card">
                <strong>Return to dashboard for launch posture</strong>
                <p>After redeeming or evaluating an offer, verify that readiness, quota, and action queue all move into a safer state.</p>
                <InlineButton onClick={() => onNavigate('dashboard')} tone="secondary">
                  Open dashboard
                </InlineButton>
              </article>
            </div>
          </Surface>
        </TabsContent>

        <TabsContent className="space-y-6" value="ledger">
          <Surface detail="Ledger entries are sourced from the live portal billing boundary." title="Points ledger">
            {ledger.length ? (
              <DataTable
                columns={[
                  { key: 'units', label: 'Units', render: (row) => formatUnits(row.units) },
                  { key: 'amount', label: 'Amount', render: (row) => formatCurrency(row.amount) },
                  { key: 'project', label: 'Project', render: (row) => row.project_id },
                ]}
                empty="No points ledger entries recorded yet."
                getKey={(row, index) => `${row.project_id}-${row.units}-${index}`}
                rows={ledger}
              />
            ) : (
              <EmptyState
                detail="Once requests are billed, the points ledger will provide a transaction-style view here."
                title="No ledger entries yet"
              />
            )}
          </Surface>

          <Surface detail="Use the ledger and request demand together before choosing the next quota action." title="Next move">
            <div className="portalx-checklist-grid">
              <article className="portalx-checklist-card">
                <strong>Inspect usage before choosing the next commercial step</strong>
                <p>Use request history and token burn to decide whether a coupon, a pack, or a subscription is the right move.</p>
                <InlineButton onClick={() => onNavigate('usage')} tone="ghost">
                  Open usage
                </InlineButton>
              </article>
            </div>
          </Surface>
        </TabsContent>
      </Tabs>
    </>
  );
}
