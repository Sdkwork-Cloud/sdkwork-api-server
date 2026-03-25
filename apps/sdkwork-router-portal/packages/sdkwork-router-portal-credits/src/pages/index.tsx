import { useDeferredValue, useEffect, useMemo, useState } from 'react';
import type { FormEvent, ReactNode } from 'react';
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
  Input,
  InlineButton,
  Select,
  ToolbarField,
  ToolbarDisclosure,
  ToolbarSearchField,
  usePortalI18n,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type { CouponOffer, LedgerEntry, ProjectBillingSummary } from 'sdkwork-router-portal-types';

import { CouponImpactCard } from '../components';
import { loadCreditsPageData } from '../repository';
import {
  buildCouponImpactPreview,
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

type CreditsTableMode = 'offers' | 'ledger';

type OfferRow = CouponOffer & {
  kind: 'offers';
};

type LedgerRow = LedgerEntry & {
  kind: 'ledger';
};

type CreditsTableRow = OfferRow | LedgerRow;

function creditsRowKey(row: CreditsTableRow, index: number): string {
  if (row.kind === 'offers') {
    return row.code;
  }

  return `${row.project_id}-${row.units}-${row.amount}-${index}`;
}

export function PortalCreditsPage({ onNavigate }: PortalCreditsPageProps) {
  const { t } = usePortalI18n();
  const [summary, setSummary] = useState<ProjectBillingSummary>(emptySummary);
  const [ledger, setLedger] = useState<LedgerEntry[]>([]);
  const [couponCode, setCouponCode] = useState('');
  const [selectedOffer, setSelectedOffer] = useState<CouponOffer | null>(null);
  const [couponStatus, setCouponStatus] = useState('Redeem workspace offers and keep points posture visible before traffic spikes.');
  const [status, setStatus] = useState('Loading points posture...');
  const [redeemDialogOpen, setRedeemDialogOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [tableMode, setTableMode] = useState<CreditsTableMode>('offers');
  const deferredSearch = useDeferredValue(searchQuery.trim().toLowerCase());

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

  const couponPreview = useMemo(
    () => (selectedOffer ? buildCouponImpactPreview(summary, selectedOffer) : null),
    [selectedOffer, summary],
  );
  const offers = listCouponOffers();
  const visibleOffers = offers.filter((offer) =>
    !deferredSearch
    || [offer.code, offer.title, offer.benefit, offer.description]
      .join(' ')
      .toLowerCase()
      .includes(deferredSearch));
  const visibleLedger = ledger.filter((row) =>
    !deferredSearch
    || row.project_id.toLowerCase().includes(deferredSearch));

  let rows: CreditsTableRow[] = visibleOffers.map((offer) => ({
    ...offer,
    kind: 'offers',
  }));
  let columns: Array<{ key: string; label: string; render: (row: CreditsTableRow) => ReactNode }> = [
    { key: 'code', label: 'Code', render: (row) => row.kind === 'offers' ? row.code : '-' },
    { key: 'title', label: 'Offer', render: (row) => row.kind === 'offers' ? row.title : '-' },
    { key: 'benefit', label: 'Benefit', render: (row) => row.kind === 'offers' ? row.benefit : '-' },
    { key: 'description', label: 'Best for', render: (row) => row.kind === 'offers' ? row.description : '-' },
    {
      key: 'actions',
      label: 'Actions',
      render: (row) => row.kind === 'offers' ? (
        <button
          type="button"
          onClick={() => previewOffer(row)}
          className="inline-flex h-9 items-center justify-center rounded-2xl border border-zinc-200 bg-white px-3 text-sm font-medium text-zinc-600 transition hover:bg-zinc-100 hover:text-zinc-950 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300 dark:hover:bg-zinc-900 dark:hover:text-zinc-50"
        >
          {t('Preview offer')}
        </button>
      ) : '-',
    },
  ];
  let emptyTitle = t('No offers for this slice');
  let emptyDetail = offers.length
    ? 'Adjust the search to reveal a different seeded coupon offer.'
    : status;
  let emptyText = t('No offers match the current filter.');

  if (tableMode === 'ledger') {
    rows = visibleLedger.map((entry) => ({
      ...entry,
      kind: 'ledger',
    }));
    columns = [
      { key: 'project', label: 'Project', render: (row) => row.kind === 'ledger' ? row.project_id : '-' },
      { key: 'units', label: 'Units', render: (row) => row.kind === 'ledger' ? String(row.units) : '-' },
      { key: 'amount', label: 'Amount', render: (row) => row.kind === 'ledger' ? `$${row.amount.toFixed(2)}` : '-' },
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
              hint={t('Seed coupons can be replaced by a live redemption backend without changing the page contract.')}
              label={t('Coupon code')}
            >
              <Input
                onChange={(event) => setCouponCode(event.target.value)}
                placeholder={t('WELCOME100')}
                value={couponCode}
              />
            </FormField>
            {couponPreview ? <CouponImpactCard preview={couponPreview} /> : null}
            <DialogFooter>
              <Button onClick={() => setRedeemDialogOpen(false)} type="button" variant="ghost">
                {t('Close')}
              </Button>
              <Button type="submit">{t('Preview redemption')}</Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <div className="grid gap-4">
        <section
          data-slot="portal-credits-toolbar"
          className="rounded-[28px] border border-zinc-200/80 bg-white/92 p-4 shadow-[0_18px_48px_rgba(15,23,42,0.08)] backdrop-blur dark:border-zinc-800/80 dark:bg-zinc-950/70 sm:p-5"
        >
          <div className="flex flex-col gap-3">
            <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
              <div className="flex flex-wrap items-center gap-3">
                <Button type="button" onClick={() => setRedeemDialogOpen(true)}>
                  {t('Redeem credits')}
                </Button>
                <InlineButton onClick={() => onNavigate('billing')} tone="secondary">
                  {t('Review billing')}
                </InlineButton>
                <InlineButton onClick={() => onNavigate('usage')} tone="secondary">
                  {t('Open usage')}
                </InlineButton>
              </div>

              <ToolbarSearchField
                label={t('Search offers or ledger')}
                value={searchQuery}
                onChange={(event) => setSearchQuery(event.target.value)}
                placeholder={t('Search offers or ledger')}
                className="w-full lg:max-w-[24rem]"
              />
            </div>

            <ToolbarDisclosure>
              <ToolbarField label={t('View mode')} className="w-full sm:max-w-[18rem]">
                  <Select
                    value={tableMode}
                    onChange={(event) => setTableMode(event.target.value as CreditsTableMode)}
                  >
                    <option value="offers">{t('Offers')}</option>
                    <option value="ledger">{t('Ledger')}</option>
                  </Select>
              </ToolbarField>
            </ToolbarDisclosure>
          </div>
        </section>

        {rows.length ? (
          <DataTable
            columns={columns}
            empty={emptyText}
            getKey={creditsRowKey}
            rows={rows}
          />
        ) : (
          <EmptyState
            detail={emptyDetail}
            title={emptyTitle}
          />
        )}
      </div>
    </>
  );
}
