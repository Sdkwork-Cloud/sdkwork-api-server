use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use sdkwork_api_app_catalog::{
    canonical_catalog_pricing_plan_code, normalize_commercial_pricing_plan_code,
};
use sdkwork_api_app_commerce::{
    create_admin_commerce_reconciliation_run, create_admin_commerce_refund,
    current_canonical_commercial_catalog_for_store, delete_admin_payment_method,
    list_admin_commerce_reconciliation_items, list_admin_commerce_reconciliation_runs,
    list_admin_commerce_refunds_for_order, list_admin_commerce_webhook_delivery_attempts,
    list_admin_commerce_webhook_inbox, list_admin_payment_method_credential_bindings,
    list_admin_payment_methods, list_payment_attempts_for_order, persist_admin_payment_method,
    replace_admin_payment_method_credential_bindings, AdminCommerceReconciliationRunCreateRequest,
    AdminCommerceRefundCreateRequest,
};
use sdkwork_api_app_marketing::load_marketing_order_evidence;
use sdkwork_api_domain_billing::{PricingPlanRecord, PricingRateRecord};
use sdkwork_api_domain_catalog::{
    ApiProduct, CatalogPublication, CatalogPublicationLifecycleAction,
    CatalogPublicationLifecycleAuditOutcome, CatalogPublicationLifecycleAuditRecord,
    CatalogPublicationStatus, ProductOffer,
};
use sdkwork_api_domain_commerce::{
    CommerceOrderRecord, CommercePaymentAttemptRecord, CommercePaymentEventRecord,
    CommerceReconciliationItemRecord, CommerceReconciliationRunRecord, CommerceRefundRecord,
    CommerceWebhookDeliveryAttemptRecord, CommerceWebhookInboxRecord,
    PaymentMethodCredentialBindingRecord, PaymentMethodRecord,
};
use sdkwork_api_domain_marketing::{
    CouponCodeRecord, CouponRedemptionRecord, CouponReservationRecord, CouponRollbackRecord,
    CouponTemplateRecord, MarketingCampaignRecord,
};
use sdkwork_api_observability::RequestId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

use crate::{
    admin_commerce_error_response, commercial_billing_error_response, commercial_billing_kernel,
    error_response, synchronize_due_pricing_plan_lifecycle, unix_timestamp_ms, AdminApiState,
    AuthenticatedAdminClaims, ErrorResponse, PublishCommercialCatalogPublicationRequest,
    RetireCommercialCatalogPublicationRequest, ScheduleCommercialCatalogPublicationRequest,
};

mod order;
mod payment;
mod publication;
mod reconciliation;

pub(crate) use order::{
    get_commerce_order_audit_handler, list_recent_commerce_orders_handler, CommerceOrderAuditRecord,
};
pub(crate) use payment::{
    create_commerce_refund_handler, delete_payment_method_handler,
    list_commerce_payment_attempts_handler, list_commerce_payment_events_handler,
    list_commerce_refunds_handler, list_payment_method_credential_bindings_handler,
    list_payment_methods_handler, put_payment_method_handler,
    replace_payment_method_credential_bindings_handler,
};
pub(crate) use publication::{
    get_commercial_catalog_publication_handler, list_commercial_catalog_publications_handler,
    publish_commercial_catalog_publication_handler, retire_commercial_catalog_publication_handler,
    schedule_commercial_catalog_publication_handler, CommercialCatalogPublicationDetail,
    CommercialCatalogPublicationMutationResult, CommercialCatalogPublicationProjection,
};
pub(crate) use reconciliation::{
    create_commerce_reconciliation_run_handler, list_commerce_reconciliation_items_handler,
    list_commerce_reconciliation_runs_handler, list_commerce_webhook_delivery_attempts_handler,
    list_commerce_webhook_inbox_handler,
};
