import { useDeferredValue, useEffect, useState } from 'react';
import type { FormEvent, ReactNode } from 'react';
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
  Input,
  InlineButton,
  MetricCard,
  Select,
  ToolbarField,
  ToolbarInline,
  ToolbarSearchField,
  usePortalI18n,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type {
  LedgerEntry,
  PortalCommerceCoupon,
  ProjectBillingSummary,
} from 'sdkwork-router-portal-types';

import { CouponImpactCard } from '../components';
import {
  createCreditsCouponRedemption,
  loadCreditsPageData,
  previewCreditsCouponRedemption,
} from '../repository';
import { buildCouponImpactPreview, recommendCouponOffer } from '../services';
import type { CouponImpactPreview, PortalCreditsPageProps } from '../types';

const emptySummary: ProjectBillingSummary = {
  project_id: '',
  entry_count: 0,
  used_units: 0,
  booked_amount: 0,
  exhausted: false,
};

type CreditsTableMode = 'offers' | 'ledger';
type OfferStateFilter = 'all' | 'eligible' | 'expiring' | 'archived';

type OfferRow = PortalCommerceCoupon & {
  kind: 'offers';
};

type LedgerRow = LedgerEntry & {
  kind: 'ledger';
};

type CreditsTableRow = OfferRow | LedgerRow;

const EXPIRING_SOON_WINDOW_DAYS = 14;

function creditsRowKey(row: CreditsTableRow, index: number): string {
  if (row.kind === 'offers') {
    return row.code;
  }

  return `${row.project_id}-${row.units}-${row.amount}-${index}`;
}

function formatQuotaPressure(summary: ProjectBillingSummary): string {
  if (summary.exhausted) {
    return 'High';
  }

  if (summary.remaining_units === null || summary.remaining_units === undefined) {
    return 'Stable';
  }

  if (summary.remaining_units <= Math.max(1_000, summary.used_units * 0.1)) {
    return 'Watch';
  }

  return 'Stable';
}

function daysUntilCouponExpiry(expiresOn: string): number | null {
  const expiryValue = Date.parse(expiresOn);
  if (Number.isNaN(expiryValue)) {
    return null;
  }

  const now = new Date();
  const startOfTodayUtc = Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate());
  return Math.ceil((expiryValue - startOfTodayUtc) / 86_400_000);
}

function isOfferExpiringSoon(coupon: PortalCommerceCoupon): boolean {
  const days = daysUntilCouponExpiry(coupon.expires_on);
  return coupon.active && days !== null && days >= 0 && days <= EXPIRING_SOON_WINDOW_DAYS;
}

function offerStateLabel(coupon: PortalCommerceCoupon): string {
  if (!coupon.active) {
    return 'Archived';
  }

  if (isOfferExpiringSoon(coupon)) {
    return 'Expiring soon';
  }

  return 'Eligible';
}

export function PortalCreditsPage({ onNavigate }: PortalCreditsPageProps) {
  const { t } = usePortalI18n();
  const [summary, setSummary] = useState<ProjectBillingSummary>(emptySummary);
  const [ledger, setLedger] = useState<LedgerEntry[]>([]);
  const [coupons, setCoupons] = useState<PortalCommerceCoupon[]>([]);
  const [couponCode, setCouponCode] = useState('');
  const [selectedOffer, setSelectedOffer] = useState<PortalCommerceCoupon | null>(null);
  const [couponPreview, setCouponPreview] = useState<CouponImpactPreview | null>(null);
  const [couponStatus, setCouponStatus] = useState(
    'Redeem workspace offers and keep points posture visible before traffic spikes.',
  );
  const [status, setStatus] = useState('Loading points posture...');
  const [redeemDialogOpen, setRedeemDialogOpen] = useState(false);
  const [redeemLoading, setRedeemLoading] = useState(false);
  const [redeemAction, setRedeemAction] = useState<'preview' | 'submit'>('preview');
  const [searchQuery, setSearchQuery] = useState('');
  const [tableMode, setTableMode] = useState<CreditsTableMode>('offers');
  const [offerState, setOfferState] = useState<OfferStateFilter>('all');
  const deferredSearch = useDeferredValue(searchQuery.trim().toLowerCase());

  useEffect(() => {
    let cancelled = false;

    void refreshCreditsPage({
      nextStatus: 'Live billing data is mapped into a points-oriented portal view.',
      cancelled: () => cancelled,
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

  async function refreshCreditsPage(options?: {
    preferredCode?: string;
    nextStatus?: string;
    cancelled?: () => boolean;
  }) {
    const data = await loadCreditsPageData();
    if (options?.cancelled?.()) {
      return;
    }

    setSummary(data.summary);
    setLedger(data.ledger);
    setCoupons(data.coupons);
    const nextOffer =
      data.coupons.find((coupon) => coupon.code === options?.preferredCode)
      ?? recommendCouponOffer(data.summary, data.coupons);
    if (nextOffer) {
      setSelectedOffer(nextOffer);
      setCouponCode(nextOffer.code);
    } else {
      setSelectedOffer(null);
      setCouponCode('');
    }
    if (options?.nextStatus) {
      setStatus(options.nextStatus);
    }
  }

  async function loadCouponPreview(offer: PortalCommerceCoupon) {
    setSelectedOffer(offer);
    setCouponPreview(null);
    setRedeemAction('preview');
    setRedeemLoading(true);
    setCouponStatus(`Loading a live redemption preview for ${offer.code}...`);

    try {
      const quote = await previewCreditsCouponRedemption({
        target_id: offer.code,
        current_remaining_units: summary.remaining_units ?? null,
      });
      setCouponPreview(buildCouponImpactPreview(offer, quote));
      setRedeemAction('submit');
      setCouponStatus(
        `${offer.code} is priced by the live commerce quote service and ready to redeem now.`,
      );
    } catch (error) {
      setCouponPreview(null);
      setRedeemAction('preview');
      setCouponStatus(portalErrorMessage(error));
    } finally {
      setRedeemLoading(false);
    }
  }

  async function submitCouponRedemption(offer: PortalCommerceCoupon) {
    setRedeemLoading(true);
    setCouponStatus(`Redeeming ${offer.code} for this workspace...`);

    try {
      await createCreditsCouponRedemption({
        target_id: offer.code,
      });
      setCouponPreview(null);
      setRedeemAction('preview');
      setRedeemDialogOpen(false);
      await refreshCreditsPage({
        preferredCode: offer.code,
        nextStatus: `${offer.code} was redeemed and the workspace credit posture was refreshed.`,
      });
    } catch (error) {
      setCouponStatus(portalErrorMessage(error));
    } finally {
      setRedeemLoading(false);
    }
  }

  function handleCouponSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const normalizedCode = couponCode.trim().toUpperCase();
    const offer = coupons.find((coupon) => coupon.code === normalizedCode);
    if (!offer) {
      setCouponPreview(null);
      setSelectedOffer(null);
      setRedeemAction('preview');
      setCouponStatus('Coupon code not recognized in the current portal commerce catalog.');
      return;
    }

    if (redeemAction === 'submit' && selectedOffer?.code === offer.code && couponPreview) {
      void submitCouponRedemption(offer);
      return;
    }

    void loadCouponPreview(offer);
  }

  function previewOffer(offer: PortalCommerceCoupon) {
    setCouponCode(offer.code);
    setRedeemDialogOpen(true);
    setRedeemAction('preview');
    void loadCouponPreview(offer);
  }

  const visibleOffers = coupons.filter(
    (offer) =>
      (
        offerState === 'all'
        || (offerState === 'eligible' && offer.active)
        || (offerState === 'expiring' && isOfferExpiringSoon(offer))
        || (offerState === 'archived' && !offer.active)
      )
      && (
        !deferredSearch
        || [
          offer.code,
          offer.discount_label,
          offer.note,
          offer.audience,
          offer.expires_on,
        ]
          .join(' ')
          .toLowerCase()
          .includes(deferredSearch)
      ),
  );
  const visibleLedger = ledger.filter(
    (row) => !deferredSearch || row.project_id.toLowerCase().includes(deferredSearch),
  );
  const totalBonusUnits = coupons.reduce((sum, coupon) => sum + coupon.bonus_units, 0);
  const eligibleOffers = coupons.filter((coupon) => coupon.active).length;
  const quotaPressure = formatQuotaPressure(summary);
  const primaryRedeemLabel = redeemLoading
    ? redeemAction === 'submit'
      ? t('Redeeming...')
      : t('Loading preview...')
    : redeemAction === 'submit'
      ? t('Redeem now')
      : t('Preview redemption');

  let rows: CreditsTableRow[] = visibleOffers.map((offer) => ({
    ...offer,
    kind: 'offers',
  }));
  let columns: Array<{ key: string; label: string; render: (row: CreditsTableRow) => ReactNode }> =
    [
      { key: 'code', label: 'Code', render: (row) => (row.kind === 'offers' ? row.code : '-') },
      {
        key: 'benefit',
        label: 'Benefit',
        render: (row) => (row.kind === 'offers' ? row.discount_label : '-'),
      },
      {
        key: 'bonus',
        label: 'Bonus units',
        render: (row) =>
          row.kind === 'offers' ? String(row.bonus_units.toLocaleString()) : '-',
      },
      {
        key: 'state',
        label: 'Offer state',
        render: (row) =>
          row.kind === 'offers' ? (
            <div className="flex flex-col gap-1">
              <strong className="text-zinc-950 dark:text-zinc-50">
                {offerStateLabel(row)}
              </strong>
              <span className="text-xs text-zinc-500 dark:text-zinc-400">
                Expires {row.expires_on}
              </span>
            </div>
          ) : '-',
      },
      {
        key: 'description',
        label: 'Best for',
        render: (row) => (row.kind === 'offers' ? row.note : '-'),
      },
      {
        key: 'actions',
        label: 'Actions',
        render: (row) =>
          row.kind === 'offers' ? (
            <Button
              type="button"
              onClick={() => previewOffer(row)}
              className="inline-flex h-9 items-center justify-center rounded-2xl border border-zinc-200 bg-white px-3 text-sm font-medium text-zinc-600 transition hover:bg-zinc-100 hover:text-zinc-950 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300 dark:hover:bg-zinc-900 dark:hover:text-zinc-50"
              variant="secondary"
            >
              {t('Preview offer')}
            </Button>
          ) : (
            '-'
          ),
      },
    ];
  let emptyTitle = t('No offers for this slice');
  let emptyDetail = coupons.length
    ? 'Adjust the search or Offer state to reveal a different live coupon offer.'
    : status;
  let emptyText = t('No offers match the current filter.');

  if (tableMode === 'ledger') {
    rows = visibleLedger.map((entry) => ({
      ...entry,
      kind: 'ledger',
    }));
    columns = [
      {
        key: 'project',
        label: 'Project',
        render: (row) => (row.kind === 'ledger' ? row.project_id : '-'),
      },
      {
        key: 'units',
        label: 'Units',
        render: (row) => (row.kind === 'ledger' ? String(row.units) : '-'),
      },
      {
        key: 'amount',
        label: 'Amount',
        render: (row) => (row.kind === 'ledger' ? `$${row.amount.toFixed(2)}` : '-'),
      },
    ];
    emptyTitle = t('No ledger entries yet');
    emptyDetail = ledger.length
      ? 'Once requests are billed, the points ledger will provide a transaction-style view here.'
      : status;
    emptyText = t('No points ledger entries recorded yet.');
  }

  return (
    <>
      <Dialog open={redeemDialogOpen} onOpenChange={setRedeemDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('Redeem credits')}</DialogTitle>
            <DialogDescription>{couponStatus}</DialogDescription>
          </DialogHeader>

          <form className="grid gap-4" onSubmit={handleCouponSubmit}>
            <FormField
              hint={t(
                'Coupon previews are validated by the portal backend rather than by seeded UI logic.',
              )}
              label={t('Coupon code')}
            >
              <Input
                onChange={(event) => {
                  setCouponCode(event.target.value);
                  setCouponPreview(null);
                  setRedeemAction('preview');
                }}
                placeholder={t('WELCOME100')}
                value={couponCode}
              />
            </FormField>
            {couponPreview ? <CouponImpactCard preview={couponPreview} /> : null}
            <DialogFooter>
              <Button onClick={() => setRedeemDialogOpen(false)} type="button" variant="ghost">
                {t('Close')}
              </Button>
              <Button disabled={redeemLoading} type="submit">
                {primaryRedeemLabel}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <div className="grid gap-4">
        <section
          data-slot="portal-credits-toolbar"
          className="rounded-[28px] border border-zinc-200/80 bg-white/92 p-4 shadow-[0_18px_48px_rgba(15,23,42,0.08)] backdrop-blur dark:border-zinc-800/80 dark:bg-zinc-950/70 sm:p-5"
        >
          <ToolbarInline>
            <ToolbarSearchField
              label={t('Search offers or ledger')}
              value={searchQuery}
              onChange={(event) => setSearchQuery(event.target.value)}
              placeholder={t('Search offers or ledger')}
              className="min-w-[15rem] flex-[0_1_20rem]"
            />
            <ToolbarField label={t('View mode')} className="min-w-[10rem] shrink-0">
              <Select
                value={tableMode}
                onChange={(event) => setTableMode(event.target.value as CreditsTableMode)}
              >
                <option value="offers">{t('Offers')}</option>
                <option value="ledger">{t('Ledger')}</option>
              </Select>
            </ToolbarField>
            <ToolbarField label={t('Offer state')} className="min-w-[12rem] shrink-0">
              <Select
                value={offerState}
                onChange={(event) => setOfferState(event.target.value as OfferStateFilter)}
                disabled={tableMode === 'ledger'}
              >
                <option value="all">{t('All offers')}</option>
                <option value="eligible">{t('Eligible offers')}</option>
                <option value="expiring">{t('Expiring soon')}</option>
                <option value="archived">{t('Archived offers')}</option>
              </Select>
            </ToolbarField>
            <div className="ml-auto flex shrink-0 items-center gap-2.5 whitespace-nowrap">
              <Button
                type="button"
                onClick={() => {
                  setRedeemDialogOpen(true);
                  if (selectedOffer) {
                    void loadCouponPreview(selectedOffer);
                  }
                }}
              >
                {t('Redeem credits')}
              </Button>
              <InlineButton onClick={() => onNavigate('billing')} tone="secondary">
                {t('Review billing')}
              </InlineButton>
              <InlineButton onClick={() => onNavigate('usage')} tone="secondary">
                {t('Open usage')}
              </InlineButton>
            </div>
          </ToolbarInline>
        </section>

        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
          <MetricCard
            label="Eligible offers"
            value={String(eligibleOffers)}
            detail="Currently active coupon offers that can be redeemed by this workspace."
          />
          <MetricCard
            label="Potential bonus units"
            value={summary.entry_count > 0 ? `${totalBonusUnits.toLocaleString()}` : `${totalBonusUnits.toLocaleString()}`}
            detail="Total bonus capacity still represented by the currently visible offer catalog."
          />
          <MetricCard
            label="Ledger entries"
            value={String(ledger.length)}
            detail="Transaction-style points evidence already recorded for this workspace."
          />
          <MetricCard
            label="Quota pressure"
            value={quotaPressure}
            detail={
              summary.remaining_units === null || summary.remaining_units === undefined
                ? 'Quota headroom is currently unbounded for this workspace slice.'
                : `${summary.remaining_units.toLocaleString()} units remain before the current quota posture tightens.`
            }
          />
        </div>

        <DataTable
          columns={columns}
          empty={(
            <div className="mx-auto flex max-w-[32rem] flex-col items-center gap-2 text-center">
              <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                {emptyTitle}
              </strong>
              <p className="text-sm text-zinc-500 dark:text-zinc-400">{emptyDetail || emptyText}</p>
            </div>
          )}
          getKey={creditsRowKey}
          rows={rows}
        />
      </div>
    </>
  );
}
