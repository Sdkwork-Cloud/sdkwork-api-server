export interface CommercePaymentAttemptRecord {
  payment_attempt_id: string;
  order_id: string;
  project_id: string;
  user_id: string;
  payment_method_id: string;
  provider: string;
  channel: string;
  status: string;
  idempotency_key: string;
  attempt_sequence: number;
  amount_minor: number;
  currency_code: string;
  captured_amount_minor: number;
  refunded_amount_minor: number;
  provider_payment_intent_id?: string | null;
  provider_checkout_session_id?: string | null;
  provider_reference?: string | null;
  checkout_url?: string | null;
  qr_code_payload?: string | null;
  request_payload_json: string;
  response_payload_json: string;
  error_code?: string | null;
  error_message?: string | null;
  initiated_at_ms: number;
  expires_at_ms?: number | null;
  completed_at_ms?: number | null;
  updated_at_ms: number;
}

export interface CommerceWebhookInboxRecord {
  webhook_inbox_id: string;
  provider: string;
  payment_method_id?: string | null;
  provider_event_id?: string | null;
  dedupe_key: string;
  signature_header?: string | null;
  payload_json: string;
  processing_status: string;
  retry_count: number;
  max_retry_count: number;
  last_error_message?: string | null;
  next_retry_at_ms?: number | null;
  first_received_at_ms: number;
  last_received_at_ms: number;
  processed_at_ms?: number | null;
}

export interface CommerceWebhookDeliveryAttemptRecord {
  delivery_attempt_id: string;
  webhook_inbox_id: string;
  processing_status: string;
  response_code?: number | null;
  error_message?: string | null;
  started_at_ms: number;
  finished_at_ms?: number | null;
}

export interface CommerceRefundRecord {
  refund_id: string;
  order_id: string;
  payment_attempt_id?: string | null;
  payment_method_id?: string | null;
  provider: string;
  provider_refund_id?: string | null;
  idempotency_key: string;
  reason?: string | null;
  status: string;
  amount_minor: number;
  currency_code: string;
  request_payload_json: string;
  response_payload_json: string;
  created_at_ms: number;
  updated_at_ms: number;
  completed_at_ms?: number | null;
}

export interface CommerceReconciliationRunRecord {
  reconciliation_run_id: string;
  provider: string;
  payment_method_id?: string | null;
  scope_started_at_ms: number;
  scope_ended_at_ms: number;
  status: string;
  summary_json: string;
  created_at_ms: number;
  updated_at_ms: number;
  completed_at_ms?: number | null;
}

export interface CommerceReconciliationItemRecord {
  reconciliation_item_id: string;
  reconciliation_run_id: string;
  order_id?: string | null;
  payment_attempt_id?: string | null;
  refund_id?: string | null;
  external_reference?: string | null;
  discrepancy_type: string;
  status: string;
  expected_amount_minor: number;
  provider_amount_minor?: number | null;
  detail_json: string;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface AdminCommerceRefundCreateRequest {
  payment_attempt_id?: string | null;
  amount_minor?: number | null;
  reason?: string | null;
  idempotency_key?: string | null;
}

export interface AdminCommerceReconciliationRunCreateRequest {
  provider: string;
  payment_method_id?: string | null;
  scope_started_at_ms: number;
  scope_ended_at_ms: number;
}
