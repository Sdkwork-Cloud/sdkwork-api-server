import { useDeferredValue, useState } from 'react';
import type { FormEvent } from 'react';

import {
  AdminDialog,
  ConfirmDialog,
  DataTable,
  Dialog,
  DialogContent,
  DialogFooter,
  DialogTrigger,
  Surface,
  FormField,
  Input,
  StatCard,
  formatAdminNumber,
  InlineButton,
  PageToolbar,
  Pill,
  Select,
  Textarea,
  ToolbarField,
  ToolbarInline,
  ToolbarSearchField,
} from 'sdkwork-router-admin-commons';
import type { AdminPageProps, CouponRecord } from 'sdkwork-router-admin-types';

type CouponsPageProps = AdminPageProps & {
  onSaveCoupon: (coupon: CouponRecord) => Promise<void> | void;
  onToggleCoupon: (coupon: CouponRecord) => Promise<void> | void;
  onDeleteCoupon: (couponId: string) => Promise<void> | void;
};

type CouponStatusFilter = 'all' | 'active' | 'at_risk' | 'archived';

const EXPIRING_SOON_WINDOW_DAYS = 14;
const LOW_QUOTA_THRESHOLD = 25;

function createEmptyCouponDraft(): CouponRecord {
  return {
    id: '',
    code: '',
    discount_label: '10% off first bill',
    audience: 'new_signup',
    remaining: 100,
    active: true,
    note: 'Launch campaign',
    expires_on: '2026-12-31',
  };
}

function daysUntilExpiry(expiresOn: string): number | null {
  const expiryValue = Date.parse(expiresOn);
  if (Number.isNaN(expiryValue)) {
    return null;
  }

  const now = new Date();
  const startOfTodayUtc = Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate());
  return Math.ceil((expiryValue - startOfTodayUtc) / 86_400_000);
}

function isCouponExpiringSoon(coupon: CouponRecord): boolean {
  if (!coupon.active) {
    return false;
  }

  const days = daysUntilExpiry(coupon.expires_on);
  return days !== null && days >= 0 && days <= EXPIRING_SOON_WINDOW_DAYS;
}

function isCouponAtRisk(coupon: CouponRecord): boolean {
  if (!coupon.active) {
    return false;
  }

  const days = daysUntilExpiry(coupon.expires_on);
  return coupon.remaining <= LOW_QUOTA_THRESHOLD || (days !== null && days <= EXPIRING_SOON_WINDOW_DAYS);
}

function quotaHealth(coupon: CouponRecord): {
  label: string;
  tone: 'default' | 'live' | 'danger';
  detail: string;
} {
  if (!coupon.active) {
    return {
      label: 'Archived',
      tone: 'default',
      detail: 'Campaign is disabled for new redemptions.',
    };
  }

  const days = daysUntilExpiry(coupon.expires_on);
  if (days !== null && days < 0) {
    return {
      label: 'Expired',
      tone: 'danger',
      detail: 'Expiry date has already passed and needs operator review.',
    };
  }

  if (coupon.remaining <= LOW_QUOTA_THRESHOLD) {
    return {
      label: 'At risk',
      tone: 'danger',
      detail: `${formatAdminNumber(coupon.remaining)} units remaining before depletion.`,
    };
  }

  if (isCouponExpiringSoon(coupon)) {
    return {
      label: 'Expiring soon',
      tone: 'danger',
      detail: `${days ?? 0} days remain before campaign expiry.`,
    };
  }

  return {
    label: 'Healthy',
    tone: 'live',
    detail: `${formatAdminNumber(coupon.remaining)} units available for redemptions.`,
  };
}

function expiryDetail(coupon: CouponRecord): string {
  const days = daysUntilExpiry(coupon.expires_on);
  if (days === null) {
    return 'Expiry date is not available.';
  }

  if (days < 0) {
    return `${Math.abs(days)} days overdue.`;
  }

  if (days === 0) {
    return 'Expires today.';
  }

  if (days <= EXPIRING_SOON_WINDOW_DAYS) {
    return `${days} days left in the current window.`;
  }

  return `${days} days of runway remain.`;
}

export function CouponsPage({
  snapshot,
  onSaveCoupon,
  onToggleCoupon,
  onDeleteCoupon,
}: CouponsPageProps) {
  const [draft, setDraft] = useState<CouponRecord>(createEmptyCouponDraft());
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<CouponStatusFilter>('all');
  const [isCouponDialogOpen, setIsCouponDialogOpen] = useState(false);
  const [pendingDeleteCoupon, setPendingDeleteCoupon] = useState<CouponRecord | null>(null);
  const deferredQuery = useDeferredValue(search.trim().toLowerCase());
  const activeCoupons = snapshot.coupons.filter((coupon) => coupon.active);
  const archivedCoupons = snapshot.coupons.filter((coupon) => !coupon.active);
  const atRiskCoupons = activeCoupons.filter(isCouponAtRisk);
  const expiringSoonCoupons = activeCoupons.filter(isCouponExpiringSoon);
  const remainingQuota = activeCoupons.reduce(
    (total, coupon) => total + Math.max(coupon.remaining, 0),
    0,
  );
  const coveredAudiences = new Set(
    activeCoupons
      .map((coupon) => coupon.audience.trim().toLowerCase())
      .filter(Boolean),
  );
  const nextExpiringCoupon =
    activeCoupons
      .map((coupon) => ({
        coupon,
        days: daysUntilExpiry(coupon.expires_on),
      }))
      .filter((item) => item.days !== null && item.days >= 0)
      .sort((left, right) => (left.days ?? Number.MAX_SAFE_INTEGER) - (right.days ?? Number.MAX_SAFE_INTEGER))[0]
      ?? null;

  const filteredCoupons = snapshot.coupons.filter((coupon) => {
    const matchesStatus = statusFilter === 'all'
      || (statusFilter === 'active' && coupon.active)
      || (statusFilter === 'archived' && !coupon.active)
      || (statusFilter === 'at_risk' && isCouponAtRisk(coupon));

    if (!matchesStatus) {
      return false;
    }

    const haystack = [
      coupon.code,
      coupon.discount_label,
      coupon.audience,
      coupon.note,
      coupon.expires_on,
    ]
      .join(' ')
      .toLowerCase();

    return haystack.includes(deferredQuery);
  });

  function resetCouponDialog() {
    setIsCouponDialogOpen(false);
    setDraft(createEmptyCouponDraft());
  }

  function openCouponDialog(coupon?: CouponRecord) {
    setDraft(coupon ? { ...coupon } : createEmptyCouponDraft());
    setIsCouponDialogOpen(true);
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveCoupon({
      ...draft,
      id: draft.id || `coupon_${Date.now().toString(16)}`,
      code: draft.code.trim().toUpperCase(),
      note: draft.note.trim(),
      audience: draft.audience.trim(),
      discount_label: draft.discount_label.trim(),
      expires_on: draft.expires_on.trim(),
    });
    resetCouponDialog();
  }

  async function handleDeleteCoupon() {
    if (!pendingDeleteCoupon) {
      return;
    }

    await onDeleteCoupon(pendingDeleteCoupon.id);
    setPendingDeleteCoupon(null);
  }

  return (
    <div className="adminx-page-grid">
      <PageToolbar
        compact
        actions={(
          <Dialog
            open={isCouponDialogOpen}
            onOpenChange={(nextOpen) => {
              if (!nextOpen) {
                resetCouponDialog();
                return;
              }
              setIsCouponDialogOpen(true);
            }}
          >
            <DialogTrigger asChild>
              <InlineButton tone="primary" onClick={() => openCouponDialog()}>
                New coupon
              </InlineButton>
            </DialogTrigger>
            <DialogContent size="medium">
              <AdminDialog
                title={draft.id ? 'Edit coupon campaign' : 'Create coupon'}
                detail="Use one modal for both launch and revision so the roster always stays primary in the workspace."
              >
                  <form className="adminx-form-grid" onSubmit={(event) => void handleSubmit(event)}>
                    <FormField label="Coupon code" hint="Stored in uppercase for consistency across support and redemption flows.">
                    <Input
                      value={draft.code}
                      onChange={(event) =>
                        setDraft((current) => ({
                          ...current,
                          code: event.target.value.toUpperCase(),
                        }))}
                      required
                    />
                  </FormField>
                  <FormField label="Discount label">
                    <Input
                      value={draft.discount_label}
                      onChange={(event) =>
                        setDraft((current) => ({
                          ...current,
                          discount_label: event.target.value,
                        }))}
                      required
                    />
                  </FormField>
                  <FormField label="Audience">
                    <Input
                      value={draft.audience}
                      onChange={(event) =>
                        setDraft((current) => ({
                          ...current,
                          audience: event.target.value,
                        }))}
                      required
                    />
                  </FormField>
                  <FormField label="Remaining quota">
                    <Input
                      value={String(draft.remaining)}
                      onChange={(event) =>
                        setDraft((current) => ({
                          ...current,
                          remaining: Number(event.target.value),
                        }))}
                      type="number"
                      min="0"
                      required
                    />
                  </FormField>
                  <FormField label="Expires on">
                    <Input
                      value={draft.expires_on}
                      onChange={(event) =>
                        setDraft((current) => ({
                          ...current,
                          expires_on: event.target.value,
                        }))}
                      type="date"
                      required
                    />
                  </FormField>
                  <FormField label="Status">
                    <Select
                      value={draft.active ? 'active' : 'archived'}
                      onChange={(event) =>
                        setDraft((current) => ({
                          ...current,
                          active: event.target.value === 'active',
                        }))}
                      >
                        <option value="active">Active</option>
                        <option value="archived">Archived</option>
                    </Select>
                  </FormField>
                  <FormField label="Operator note" className="adminx-field">
                    <Textarea
                      value={draft.note}
                      onChange={(event) =>
                        setDraft((current) => ({
                          ...current,
                          note: event.target.value,
                        }))}
                      required
                    />
                  </FormField>
                  <DialogFooter>
                    <InlineButton onClick={resetCouponDialog}>Cancel</InlineButton>
                    <InlineButton tone="primary" type="submit">
                      {draft.id ? 'Save coupon' : 'Create coupon'}
                    </InlineButton>
                  </DialogFooter>
                </form>
              </AdminDialog>
            </DialogContent>
          </Dialog>
        )}
      >
        <ToolbarInline>
          <ToolbarSearchField
            label="Search campaigns"
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="code, audience, note"
          />
          <ToolbarField label="Campaign state">
            <Select
              value={statusFilter}
              onChange={(event) => setStatusFilter(event.target.value as CouponStatusFilter)}
            >
              <option value="all">All campaigns</option>
              <option value="active">Active</option>
              <option value="at_risk">At risk</option>
              <option value="archived">Archived</option>
            </Select>
          </ToolbarField>
        </ToolbarInline>
      </PageToolbar>

      <section className="adminx-stat-grid">
        <StatCard
          label="Campaign posture"
          value={`${formatAdminNumber(activeCoupons.length)} live`}
          detail={`${formatAdminNumber(atRiskCoupons.length)} campaigns need attention and ${formatAdminNumber(archivedCoupons.length)} are archived.`}
        />
        <StatCard
          label="Audience coverage"
          value={formatAdminNumber(coveredAudiences.size)}
          detail={
            activeCoupons.length
              ? `${formatAdminNumber(activeCoupons.length)} live campaigns cover ${formatAdminNumber(coveredAudiences.size)} active audience segments.`
              : 'Launch a live campaign to create redeemable audience coverage.'
          }
        />
        <StatCard
          label="Remaining coupon quota"
          value={formatAdminNumber(remainingQuota)}
          detail={
            activeCoupons.length
              ? 'Combined redeemable quota currently available across active campaigns.'
              : 'No active coupon quota is currently allocated.'
          }
        />
        <StatCard
          label="Expiring soon"
          value={formatAdminNumber(expiringSoonCoupons.length)}
          detail={
            nextExpiringCoupon
              ? `${nextExpiringCoupon.coupon.code} is the next scheduled expiry on ${nextExpiringCoupon.coupon.expires_on}.`
              : 'No live campaign currently has a tracked expiry date.'
          }
        />
      </section>

      <Surface
        title="Campaign roster"
        detail="Keep launch posture, audience targeting, and redemption runway visible from one operator workbench."
        actions={(
          <div className="adminx-row">
            <Pill tone="default">{filteredCoupons.length} visible</Pill>
            <Pill tone={atRiskCoupons.length > 0 ? 'danger' : 'live'}>
              {formatAdminNumber(atRiskCoupons.length)} at risk
            </Pill>
          </div>
        )}
      >
        <DataTable
          columns={[
            {
              key: 'campaign',
              label: 'Campaign',
              render: (coupon) => (
                <div className="adminx-table-cell-stack">
                  <strong>{coupon.code}</strong>
                  <span>{coupon.note}</span>
                </div>
              ),
            },
            {
              key: 'offer',
              label: 'Offer',
              render: (coupon) => (
                <div className="adminx-table-cell-stack">
                  <strong>{coupon.discount_label}</strong>
                  <span>{coupon.audience}</span>
                </div>
              ),
            },
            {
              key: 'remaining',
              label: 'Remaining coupon quota',
              render: (coupon) => formatAdminNumber(coupon.remaining),
            },
            {
              key: 'quota-health',
              label: 'Quota health',
              render: (coupon) => {
                const health = quotaHealth(coupon);
                return (
                  <div className="adminx-table-cell-stack">
                    <Pill tone={health.tone}>{health.label}</Pill>
                    <span>{health.detail}</span>
                  </div>
                );
              },
            },
            {
              key: 'expires',
              label: 'Expiring soon',
              render: (coupon) => (
                <div className="adminx-table-cell-stack">
                  <strong>{coupon.expires_on}</strong>
                  <span>{expiryDetail(coupon)}</span>
                </div>
              ),
            },
            {
              key: 'status',
              label: 'Status',
              render: (coupon) => (
                <Pill tone={coupon.active ? 'live' : 'danger'}>
                  {coupon.active ? 'active' : 'archived'}
                </Pill>
              ),
            },
            {
              key: 'actions',
              label: 'Actions',
              render: (coupon) => (
                <div className="adminx-row">
                  <InlineButton onClick={() => openCouponDialog(coupon)}>Edit coupon campaign</InlineButton>
                  <InlineButton onClick={() => void onToggleCoupon(coupon)}>
                    {coupon.active ? 'Archive' : 'Restore'}
                  </InlineButton>
                  <InlineButton tone="danger" onClick={() => setPendingDeleteCoupon(coupon)}>
                    Delete
                  </InlineButton>
                </div>
              ),
            },
          ]}
          rows={filteredCoupons}
          empty="No coupons match the current filter."
          getKey={(coupon) => coupon.id}
        />
      </Surface>

      <ConfirmDialog
        open={Boolean(pendingDeleteCoupon)}
        title="Delete coupon campaign"
        detail={
          pendingDeleteCoupon
            ? `Remove ${pendingDeleteCoupon.code} from the campaign roster. This permanently deletes the offer from the admin control plane.`
            : ''
        }
        confirmLabel="Delete coupon"
        onClose={() => setPendingDeleteCoupon(null)}
        onConfirm={handleDeleteCoupon}
      />
    </div>
  );
}
