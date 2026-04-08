import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  DescriptionDetails,
  DescriptionItem,
  DescriptionList,
  DescriptionTerm,
  Drawer,
  DrawerBody,
  DrawerContent,
  DrawerDescription,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle,
  StatusBadge,
} from '@sdkwork/ui-pc-react';
import { formatAdminDateTime, useAdminI18n } from 'sdkwork-router-admin-core';
import type { CommerceOrderAuditRecord, CommerceOrderRecord, CommercePaymentEventRecord } from 'sdkwork-router-admin-types';
import { formatStatusLabel, latestObservedPaymentEvent } from './formatters';

type CommercialOrderAuditDrawerProps = {
  open: boolean;
  selectedOrderRecord: CommerceOrderRecord | null;
  selectedOrderAudit: CommerceOrderAuditRecord | null;
  selectedOrderPaymentEvents: CommercePaymentEventRecord[];
  isLoading: boolean;
  error: string | null;
  onOpenChange: (open: boolean) => void;
};

export function CommercialOrderAuditDrawer({
  open,
  selectedOrderRecord,
  selectedOrderAudit,
  selectedOrderPaymentEvents,
  isLoading,
  error,
  onOpenChange,
}: CommercialOrderAuditDrawerProps) {
  const { formatNumber, t } = useAdminI18n();
  const latestOrderPaymentEvent = latestObservedPaymentEvent(selectedOrderPaymentEvents);

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent side="right" size="xl">
        {open ? (
          <>
            <DrawerHeader>
              <div className="space-y-3">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="space-y-1">
                    <DrawerTitle>{t('Order audit detail')}</DrawerTitle>
                    <DrawerDescription>
                      {selectedOrderRecord
                        ? t('Order #{id}', { id: selectedOrderRecord.order_id })
                        : t('Loading selected order')}
                    </DrawerDescription>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    <StatusBadge
                      showIcon
                      status={selectedOrderRecord ? formatStatusLabel(selectedOrderRecord.status) : t('Loading')}
                      variant={
                        selectedOrderRecord?.status === 'fulfilled'
                          ? 'success'
                          : selectedOrderRecord?.status === 'refunded'
                            ? 'warning'
                            : 'secondary'
                      }
                    />
                    <StatusBadge
                      showIcon
                      status={latestOrderPaymentEvent ? formatStatusLabel(latestOrderPaymentEvent.provider) : t('No payment evidence')}
                      variant={latestOrderPaymentEvent ? 'secondary' : 'warning'}
                    />
                    {selectedOrderRecord?.applied_coupon_code ? (
                      <StatusBadge
                        showIcon
                        status={selectedOrderRecord.applied_coupon_code}
                        variant="secondary"
                      />
                    ) : null}
                  </div>
                </div>
              </div>
            </DrawerHeader>

            <DrawerBody className="space-y-4">
              {isLoading ? (
                <Card>
                  <CardHeader>
                    <CardTitle>{t('Loading order audit evidence')}</CardTitle>
                    <CardDescription>
                      {t('Payment, coupon, and campaign evidence is being loaded for the selected order.')}
                    </CardDescription>
                  </CardHeader>
                </Card>
              ) : null}

              {!isLoading && error ? (
                <Card>
                  <CardHeader>
                    <CardTitle>{t('Order audit detail unavailable')}</CardTitle>
                    <CardDescription>{error}</CardDescription>
                  </CardHeader>
                </Card>
              ) : null}

              {!isLoading && !error && selectedOrderRecord ? (
                <>
                  <Card>
                    <CardHeader>
                      <CardTitle>{t('Order audit detail')}</CardTitle>
                      <CardDescription>
                        {t('Commercial order, checkout, and coupon evidence stay bundled here so operators can reconstruct fulfillment and refund posture without switching modules.')}
                      </CardDescription>
                    </CardHeader>
                    <CardContent>
                      <DescriptionList columns={2}>
                        <DescriptionItem>
                          <DescriptionTerm>{t('Project')}</DescriptionTerm>
                          <DescriptionDetails>{selectedOrderRecord.project_id}</DescriptionDetails>
                        </DescriptionItem>
                        <DescriptionItem>
                          <DescriptionTerm>{t('User')}</DescriptionTerm>
                          <DescriptionDetails>{selectedOrderRecord.user_id}</DescriptionDetails>
                        </DescriptionItem>
                        <DescriptionItem>
                          <DescriptionTerm>{t('Target')}</DescriptionTerm>
                          <DescriptionDetails>{selectedOrderRecord.target_name}</DescriptionDetails>
                        </DescriptionItem>
                        <DescriptionItem>
                          <DescriptionTerm>{t('Target kind')}</DescriptionTerm>
                          <DescriptionDetails>{formatStatusLabel(selectedOrderRecord.target_kind)}</DescriptionDetails>
                        </DescriptionItem>
                        <DescriptionItem>
                          <DescriptionTerm>{t('List price')}</DescriptionTerm>
                          <DescriptionDetails>{selectedOrderRecord.list_price_label}</DescriptionDetails>
                        </DescriptionItem>
                        <DescriptionItem>
                          <DescriptionTerm>{t('Payable price')}</DescriptionTerm>
                          <DescriptionDetails>{selectedOrderRecord.payable_price_label}</DescriptionDetails>
                        </DescriptionItem>
                        <DescriptionItem>
                          <DescriptionTerm>{t('Granted units')}</DescriptionTerm>
                          <DescriptionDetails>{formatNumber(selectedOrderRecord.granted_units)}</DescriptionDetails>
                        </DescriptionItem>
                        <DescriptionItem>
                          <DescriptionTerm>{t('Bonus units')}</DescriptionTerm>
                          <DescriptionDetails>{formatNumber(selectedOrderRecord.bonus_units)}</DescriptionDetails>
                        </DescriptionItem>
                        <DescriptionItem>
                          <DescriptionTerm>{t('Coupon code')}</DescriptionTerm>
                          <DescriptionDetails>{selectedOrderRecord.applied_coupon_code ?? t('No coupon applied')}</DescriptionDetails>
                        </DescriptionItem>
                        <DescriptionItem>
                          <DescriptionTerm>{t('Observed')}</DescriptionTerm>
                          <DescriptionDetails>{formatAdminDateTime(selectedOrderRecord.updated_at_ms)}</DescriptionDetails>
                        </DescriptionItem>
                      </DescriptionList>
                    </CardContent>
                  </Card>

                  <Card>
                    <CardHeader>
                      <CardTitle>{t('Payment evidence timeline')}</CardTitle>
                      <CardDescription>
                        {t('Provider callbacks remain ordered here so operators can verify settlement, rejection, and refund sequencing for the selected order.')}
                      </CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-3">
                      {selectedOrderPaymentEvents.length ? (
                        selectedOrderPaymentEvents.map((event) => (
                          <Card className="border-[var(--sdk-color-border-subtle)] shadow-none" key={event.payment_event_id}>
                            <CardHeader className="space-y-2">
                              <div className="flex flex-wrap items-start justify-between gap-3">
                                <div className="space-y-1">
                                  <CardTitle className="text-sm">
                                    {formatStatusLabel(event.event_type)}
                                  </CardTitle>
                                  <CardDescription>
                                    {event.provider_event_id ?? t('No provider event id')}
                                  </CardDescription>
                                </div>
                                <div className="flex flex-wrap gap-2">
                                  <StatusBadge
                                    showIcon
                                    status={formatStatusLabel(event.provider)}
                                    variant="secondary"
                                  />
                                  <StatusBadge
                                    showIcon
                                    status={formatStatusLabel(event.processing_status)}
                                    variant={
                                      event.processing_status === 'processed'
                                        ? 'success'
                                        : event.processing_status === 'rejected'
                                          || event.processing_status === 'failed'
                                          ? 'danger'
                                          : 'secondary'
                                    }
                                  />
                                </div>
                              </div>
                            </CardHeader>
                            <CardContent>
                              <DescriptionList columns={2}>
                                <DescriptionItem>
                                  <DescriptionTerm>{t('Observed')}</DescriptionTerm>
                                  <DescriptionDetails>
                                    {formatAdminDateTime(event.processed_at_ms ?? event.received_at_ms)}
                                  </DescriptionDetails>
                                </DescriptionItem>
                                <DescriptionItem>
                                  <DescriptionTerm>{t('Order status after')}</DescriptionTerm>
                                  <DescriptionDetails>
                                    {event.order_status_after
                                      ? formatStatusLabel(event.order_status_after)
                                      : t('No derived order status')}
                                  </DescriptionDetails>
                                </DescriptionItem>
                                <DescriptionItem>
                                  <DescriptionTerm>{t('Payment event id')}</DescriptionTerm>
                                  <DescriptionDetails>{event.payment_event_id}</DescriptionDetails>
                                </DescriptionItem>
                                <DescriptionItem>
                                  <DescriptionTerm>{t('Dedupe key')}</DescriptionTerm>
                                  <DescriptionDetails>{event.dedupe_key}</DescriptionDetails>
                                </DescriptionItem>
                              </DescriptionList>
                              {event.processing_message ? (
                                <div className="mt-3 text-sm text-[var(--sdk-color-text-secondary)]">
                                  {event.processing_message}
                                </div>
                              ) : null}
                            </CardContent>
                          </Card>
                        ))
                      ) : (
                        <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                          {t('No payment evidence has been recorded for this order yet.')}
                        </div>
                      )}
                    </CardContent>
                  </Card>

                  <Card>
                    <CardHeader>
                      <CardTitle>{t('Coupon evidence chain')}</CardTitle>
                      <CardDescription>
                        {t('Reservation, redemption, rollback, code, template, and campaign evidence stays attached so discount posture can be audited together with payment callbacks.')}
                      </CardDescription>
                    </CardHeader>
                    <CardContent>
                      <DescriptionList columns={2}>
                        <DescriptionItem>
                          <DescriptionTerm>{t('Reservation')}</DescriptionTerm>
                          <DescriptionDetails>
                            {selectedOrderAudit?.coupon_reservation
                              ? `${selectedOrderAudit.coupon_reservation.coupon_reservation_id} (${formatStatusLabel(selectedOrderAudit.coupon_reservation.reservation_status)})`
                              : t('No reservation evidence')}
                          </DescriptionDetails>
                        </DescriptionItem>
                        <DescriptionItem>
                          <DescriptionTerm>{t('Redemption')}</DescriptionTerm>
                          <DescriptionDetails>
                            {selectedOrderAudit?.coupon_redemption
                              ? `${selectedOrderAudit.coupon_redemption.coupon_redemption_id} (${formatStatusLabel(selectedOrderAudit.coupon_redemption.redemption_status)})`
                              : t('No redemption evidence')}
                          </DescriptionDetails>
                        </DescriptionItem>
                        <DescriptionItem>
                          <DescriptionTerm>{t('Rollback count')}</DescriptionTerm>
                          <DescriptionDetails>
                            {formatNumber(selectedOrderAudit?.coupon_rollbacks.length ?? 0)}
                          </DescriptionDetails>
                        </DescriptionItem>
                        <DescriptionItem>
                          <DescriptionTerm>{t('Coupon code')}</DescriptionTerm>
                          <DescriptionDetails>
                            {selectedOrderAudit?.coupon_code?.code_value ?? t('No coupon code evidence')}
                          </DescriptionDetails>
                        </DescriptionItem>
                        <DescriptionItem>
                          <DescriptionTerm>{t('Coupon template')}</DescriptionTerm>
                          <DescriptionDetails>
                            {selectedOrderAudit?.coupon_template?.display_name ?? t('No template evidence')}
                          </DescriptionDetails>
                        </DescriptionItem>
                        <DescriptionItem>
                          <DescriptionTerm>{t('Marketing campaign')}</DescriptionTerm>
                          <DescriptionDetails>
                            {selectedOrderAudit?.marketing_campaign?.display_name ?? t('No campaign evidence')}
                          </DescriptionDetails>
                        </DescriptionItem>
                      </DescriptionList>
                    </CardContent>
                  </Card>

                  <Card>
                    <CardHeader>
                      <CardTitle>{t('Coupon rollback timeline')}</CardTitle>
                      <CardDescription>
                        {t('Rollback evidence confirms whether coupon subsidy and inventory were restored during refund handling.')}
                      </CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-3">
                      {selectedOrderAudit?.coupon_rollbacks.length ? (
                        selectedOrderAudit.coupon_rollbacks.map((rollback) => (
                          <Card className="border-[var(--sdk-color-border-subtle)] shadow-none" key={rollback.coupon_rollback_id}>
                            <CardHeader className="space-y-2">
                              <div className="flex flex-wrap items-start justify-between gap-3">
                                <div className="space-y-1">
                                  <CardTitle className="text-sm">
                                    {rollback.coupon_rollback_id}
                                  </CardTitle>
                                  <CardDescription>
                                    {formatAdminDateTime(rollback.updated_at_ms)}
                                  </CardDescription>
                                </div>
                                <div className="flex flex-wrap gap-2">
                                  <StatusBadge
                                    showIcon
                                    status={formatStatusLabel(rollback.rollback_type)}
                                    variant="warning"
                                  />
                                  <StatusBadge
                                    showIcon
                                    status={formatStatusLabel(rollback.rollback_status)}
                                    variant={
                                      rollback.rollback_status === 'completed'
                                        ? 'success'
                                        : rollback.rollback_status === 'failed'
                                          ? 'danger'
                                          : 'secondary'
                                    }
                                  />
                                </div>
                              </div>
                            </CardHeader>
                            <CardContent>
                              <DescriptionList columns={2}>
                                <DescriptionItem>
                                  <DescriptionTerm>{t('Restored budget')}</DescriptionTerm>
                                  <DescriptionDetails>{formatNumber(rollback.restored_budget_minor)}</DescriptionDetails>
                                </DescriptionItem>
                                <DescriptionItem>
                                  <DescriptionTerm>{t('Restored inventory')}</DescriptionTerm>
                                  <DescriptionDetails>{formatNumber(rollback.restored_inventory_count)}</DescriptionDetails>
                                </DescriptionItem>
                              </DescriptionList>
                            </CardContent>
                          </Card>
                        ))
                      ) : (
                        <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                          {t('No coupon rollback evidence has been recorded for this order.')}
                        </div>
                      )}
                    </CardContent>
                  </Card>
                </>
              ) : null}
            </DrawerBody>

            <DrawerFooter className="text-xs text-[var(--sdk-color-text-secondary)]">
              {t(
                'Order audit detail keeps payment callbacks and coupon lifecycle evidence scoped to the selected order so reconciliation triage stays deterministic.',
              )}
            </DrawerFooter>
          </>
        ) : null}
      </DrawerContent>
    </Drawer>
  );
}
