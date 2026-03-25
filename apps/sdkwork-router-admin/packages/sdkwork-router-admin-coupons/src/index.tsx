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
  ToolbarSearchField,
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
        <ToolbarSearchField
          label="Search campaigns"
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          placeholder="code, audience, note"
        />
      </PageToolbar>

      <section className="adminx-page-grid">
        <div className="adminx-row">
          <strong>Coupon roster</strong>
          <Pill tone="default">{filteredCoupons.length} visible</Pill>
        </div>
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
      </section>

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
