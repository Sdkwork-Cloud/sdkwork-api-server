import type {
  AdminCommerceReconciliationRunCreateRequest,
  AdminCommerceRefundCreateRequest,
  CommerceOrderAuditRecord,
  CommerceOrderRecord,
  CommercePaymentAttemptRecord,
  CommercePaymentEventRecord,
  CommerceReconciliationItemRecord,
  CommerceReconciliationRunRecord,
  CommerceRefundRecord,
  CommerceWebhookDeliveryAttemptRecord,
  CommerceWebhookInboxRecord,
  PaymentMethodCredentialBindingRecord,
  PaymentMethodRecord,
} from 'sdkwork-router-admin-types';
import {
  deleteEmpty,
  getJson,
  putJson,
  postJson,
  requiredToken,
} from './transport';

export function listRecentCommerceOrders(
  limit = 24,
  token?: string,
): Promise<CommerceOrderRecord[]> {
  return getJson<CommerceOrderRecord[]>(
    `/commerce/orders?limit=${encodeURIComponent(String(limit))}`,
    token,
  );
}

export function listCommercePaymentMethods(
  token?: string,
): Promise<PaymentMethodRecord[]> {
  return getJson<PaymentMethodRecord[]>('/commerce/payment-methods', token);
}

export function saveCommercePaymentMethod(
  paymentMethodId: string,
  input: PaymentMethodRecord,
): Promise<PaymentMethodRecord> {
  return putJson<PaymentMethodRecord, PaymentMethodRecord>(
    `/commerce/payment-methods/${encodeURIComponent(paymentMethodId)}`,
    input,
    requiredToken(),
  );
}

export function deleteCommercePaymentMethod(
  paymentMethodId: string,
): Promise<void> {
  return deleteEmpty(
    `/commerce/payment-methods/${encodeURIComponent(paymentMethodId)}`,
    requiredToken(),
  );
}

export function listCommercePaymentMethodCredentialBindings(
  paymentMethodId: string,
  token?: string,
): Promise<PaymentMethodCredentialBindingRecord[]> {
  return getJson<PaymentMethodCredentialBindingRecord[]>(
    `/commerce/payment-methods/${encodeURIComponent(paymentMethodId)}/credential-bindings`,
    requiredToken(token),
  );
}

export function replaceCommercePaymentMethodCredentialBindings(
  paymentMethodId: string,
  bindings: PaymentMethodCredentialBindingRecord[],
): Promise<PaymentMethodCredentialBindingRecord[]> {
  return putJson<
    PaymentMethodCredentialBindingRecord[],
    PaymentMethodCredentialBindingRecord[]
  >(
    `/commerce/payment-methods/${encodeURIComponent(paymentMethodId)}/credential-bindings`,
    bindings,
    requiredToken(),
  );
}

export function listCommercePaymentEvents(
  orderId: string,
  token?: string,
): Promise<CommercePaymentEventRecord[]> {
  return getJson<CommercePaymentEventRecord[]>(
    `/commerce/orders/${encodeURIComponent(orderId)}/payment-events`,
    token,
  );
}

export function listCommercePaymentAttempts(
  orderId: string,
  token?: string,
): Promise<CommercePaymentAttemptRecord[]> {
  return getJson<CommercePaymentAttemptRecord[]>(
    `/commerce/orders/${encodeURIComponent(orderId)}/payment-attempts`,
    token,
  );
}

export function listCommerceRefunds(
  orderId: string,
  token?: string,
): Promise<CommerceRefundRecord[]> {
  return getJson<CommerceRefundRecord[]>(
    `/commerce/orders/${encodeURIComponent(orderId)}/refunds`,
    token,
  );
}

export function createCommerceRefund(
  orderId: string,
  input: AdminCommerceRefundCreateRequest,
): Promise<CommerceRefundRecord> {
  return postJson<AdminCommerceRefundCreateRequest, CommerceRefundRecord>(
    `/commerce/orders/${encodeURIComponent(orderId)}/refunds`,
    input,
    requiredToken(),
  );
}

export function getCommerceOrderAudit(
  orderId: string,
  token?: string,
): Promise<CommerceOrderAuditRecord> {
  return getJson<CommerceOrderAuditRecord>(
    `/commerce/orders/${encodeURIComponent(orderId)}/audit`,
    token,
  );
}

export function listCommerceWebhookInbox(
  token?: string,
): Promise<CommerceWebhookInboxRecord[]> {
  return getJson<CommerceWebhookInboxRecord[]>('/commerce/webhook-inbox', token);
}

export function listCommerceWebhookDeliveryAttempts(
  webhookInboxId: string,
  token?: string,
): Promise<CommerceWebhookDeliveryAttemptRecord[]> {
  return getJson<CommerceWebhookDeliveryAttemptRecord[]>(
    `/commerce/webhook-inbox/${encodeURIComponent(webhookInboxId)}/delivery-attempts`,
    token,
  );
}

export function listCommerceReconciliationRuns(
  token?: string,
): Promise<CommerceReconciliationRunRecord[]> {
  return getJson<CommerceReconciliationRunRecord[]>(
    '/commerce/reconciliation-runs',
    token,
  );
}

export function createCommerceReconciliationRun(
  input: AdminCommerceReconciliationRunCreateRequest,
): Promise<CommerceReconciliationRunRecord> {
  return postJson<
    AdminCommerceReconciliationRunCreateRequest,
    CommerceReconciliationRunRecord
  >('/commerce/reconciliation-runs', input, requiredToken());
}

export function listCommerceReconciliationItems(
  reconciliationRunId: string,
  token?: string,
): Promise<CommerceReconciliationItemRecord[]> {
  return getJson<CommerceReconciliationItemRecord[]>(
    `/commerce/reconciliation-runs/${encodeURIComponent(reconciliationRunId)}/items`,
    token,
  );
}
