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
  FormField,
  InlineButton,
  PageToolbar,
  Pill,
  StatCard,
  Surface,
} from 'sdkwork-router-admin-commons';
import type { AdminPageProps, CouponRecord } from 'sdkwork-router-admin-types';

type CouponsPageProps = AdminPageProps & {
  onSaveCoupon: (coupon: CouponRecord) => Promise<void> | void;
  onToggleCoupon: (coupon: CouponRecord) => Promise<void> | void;
  onDeleteCoupon: (couponId: string) => Promise<void> | void;
};

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

function expiringSoonCount(coupons: CouponRecord[]): number {
  const threshold = new Date('2026-04-30').getTime();

  return coupons.filter((coupon) => {
    const expiresAt = Date.parse(coupon.expires_on);
    return Number.isFinite(expiresAt) && expiresAt <= threshold && coupon.active;
  }).length;
}

export function CouponsPage({
  snapshot,
  onSaveCoupon,
  onToggleCoupon,
  onDeleteCoupon,
}: CouponsPageProps) {
  const [draft, setDraft] = useState<CouponRecord>(createEmptyCouponDraft());
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<'all' | 'active' | 'archived'>('all');
  const [isCouponDialogOpen, setIsCouponDialogOpen] = useState(false);
  const [pendingDeleteCoupon, setPendingDeleteCoupon] = useState<CouponRecord | null>(null);
  const deferredQuery = useDeferredValue(search.trim().toLowerCase());

  const filteredCoupons = snapshot.coupons.filter((coupon) => {
    const matchesStatus = statusFilter === 'all'
      || (statusFilter === 'active' && coupon.active)
      || (statusFilter === 'archived' && !coupon.active);

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

  const activeCoupons = snapshot.coupons.filter((coupon) => coupon.active).length;
  const archivedCoupons = snapshot.coupons.length - activeCoupons;
  const totalRemaining = snapshot.coupons.reduce((sum, coupon) => sum + coupon.remaining, 0);
  const soonToExpire = expiringSoonCount(snapshot.coupons);

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
      <section className="adminx-stat-grid">
        <StatCard
          label="Campaigns"
          value={String(snapshot.coupons.length)}
          detail="Total coupon campaigns currently tracked in the control plane."
        />
        <StatCard
          label="Active campaigns"
          value={String(activeCoupons)}
          detail="Offers still eligible for redemption right now."
        />
        <StatCard
          label="Archived campaigns"
          value={String(archivedCoupons)}
          detail="Retired offers preserved for operator history and audit."
        />
        <StatCard
          label="Remaining quota"
          value={String(totalRemaining)}
          detail="Aggregate redemption inventory across all visible campaigns."
        />
        <StatCard
          label="Expiring soon"
          value={String(soonToExpire)}
          detail="Active campaigns approaching their current expiration window."
        />
      </section>

      <PageToolbar
        title="Campaign roster workbench"
        detail="Review the roster, narrow by status, then open a focused dialog when you need to launch or revise a promotion."
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
                    <input
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
                    <input
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
                    <input
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
                    <input
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
                    <input
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
                    <select
                      value={draft.active ? 'active' : 'archived'}
                      onChange={(event) =>
                        setDraft((current) => ({
                          ...current,
                          active: event.target.value === 'active',
                        }))}
                    >
                      <option value="active">Active</option>
                      <option value="archived">Archived</option>
                    </select>
                  </FormField>
                  <FormField label="Operator note" className="adminx-field">
                    <textarea
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
        <div className="adminx-form-grid">
          <FormField label="Search campaigns">
            <input
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="code, audience, note"
            />
          </FormField>
          <FormField label="Status">
            <select
              value={statusFilter}
              onChange={(event) =>
                setStatusFilter(event.target.value as 'all' | 'active' | 'archived')}
            >
              <option value="all">All campaigns</option>
              <option value="active">Active only</option>
              <option value="archived">Archived only</option>
            </select>
          </FormField>
          <div className="adminx-note">
            <strong>Campaign hygiene</strong>
            <p>
              Archiving preserves history while taking the offer offline. Deletion removes the
              campaign entirely, so use it only when support no longer needs the record.
            </p>
          </div>
        </div>
      </PageToolbar>

      <Surface
        title="Coupon roster"
        detail="Operate live coupon campaigns from one registry with clear edit, archive, restore, and delete actions."
        actions={<Pill tone="default">{filteredCoupons.length} visible</Pill>}
      >
        <DataTable
          columns={[
            {
              key: 'code',
              label: 'Code',
              render: (coupon) => (
                <div className="adminx-table-cell-stack">
                  <strong>{coupon.code}</strong>
                  <span>{coupon.note}</span>
                </div>
              ),
            },
            { key: 'discount', label: 'Discount', render: (coupon) => coupon.discount_label },
            { key: 'audience', label: 'Audience', render: (coupon) => coupon.audience },
            { key: 'remaining', label: 'Remaining', render: (coupon) => coupon.remaining },
            { key: 'expires', label: 'Expires', render: (coupon) => coupon.expires_on },
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

      <Surface
        title="Campaign playbook"
        detail="Compact operating guidance keeps the roster focused while still exposing the product rules behind healthy coupon management."
      >
        <div className="adminx-card-grid">
          <article className="adminx-mini-card">
            <div className="adminx-row">
              <strong>Use one audience per offer</strong>
              <Pill tone="seed">clarity</Pill>
            </div>
            <p>Targeting stays easier to explain when each coupon has a single audience and a single note about intent.</p>
          </article>
          <article className="adminx-mini-card">
            <div className="adminx-row">
              <strong>Keep quota visible</strong>
              <Pill tone="live">control</Pill>
            </div>
            <p>Remaining inventory belongs in the roster so support can answer campaign availability without opening the editor.</p>
          </article>
          <article className="adminx-mini-card">
            <div className="adminx-row">
              <strong>Archive before delete</strong>
              <Pill tone="default">audit</Pill>
            </div>
            <p>Archive old promotions when history still matters, and reserve deletion for mistaken or fully disposable campaigns.</p>
          </article>
        </div>
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
