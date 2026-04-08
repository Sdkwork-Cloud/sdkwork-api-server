use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use sdkwork_api_app_commerce::{
    AdminCommerceReconciliationRunCreateRequest, AdminCommerceRefundCreateRequest,
    create_admin_commerce_reconciliation_run, create_admin_commerce_refund,
    delete_admin_payment_method, list_admin_commerce_reconciliation_items,
    list_admin_commerce_reconciliation_runs, list_admin_commerce_refunds_for_order,
    list_admin_commerce_webhook_delivery_attempts, list_admin_commerce_webhook_inbox,
    list_admin_payment_method_credential_bindings, list_admin_payment_methods,
    list_payment_attempts_for_order, persist_admin_payment_method,
    replace_admin_payment_method_credential_bindings,
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
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    AdminApiState, AuthenticatedAdminClaims, ErrorResponse, admin_commerce_error_response,
    error_response,
};

#[derive(Debug, Deserialize)]
pub(crate) struct RecentCommerceOrdersQuery {
    #[serde(default)]
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CommerceOrderAuditRecord {
    pub(crate) order: CommerceOrderRecord,
    pub(crate) payment_events: Vec<CommercePaymentEventRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) coupon_reservation: Option<CouponReservationRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) coupon_redemption: Option<CouponRedemptionRecord>,
    #[serde(default)]
    pub(crate) coupon_rollbacks: Vec<CouponRollbackRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) coupon_code: Option<CouponCodeRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) coupon_template: Option<CouponTemplateRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) marketing_campaign: Option<MarketingCampaignRecord>,
}

fn clamp_recent_commerce_orders_limit(limit: Option<usize>) -> usize {
    match limit {
        Some(limit) if limit > 0 => limit.min(100),
        _ => 24,
    }
}

pub(crate) async fn list_recent_commerce_orders_handler(
    _claims: AuthenticatedAdminClaims,
    Query(query): Query<RecentCommerceOrdersQuery>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommerceOrderRecord>>, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .list_recent_commerce_orders(clamp_recent_commerce_orders_limit(query.limit))
        .await
        .map(Json)
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load recent commerce orders: {error}"),
            )
        })
}

pub(crate) async fn list_payment_methods_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<PaymentMethodRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_admin_payment_methods(state.store.as_ref())
        .await
        .map(Json)
        .map_err(admin_commerce_error_response)
}

pub(crate) async fn put_payment_method_handler(
    _claims: AuthenticatedAdminClaims,
    Path(payment_method_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(mut payment_method): Json<PaymentMethodRecord>,
) -> Result<Json<PaymentMethodRecord>, (StatusCode, Json<ErrorResponse>)> {
    let normalized_payment_method_id = payment_method_id.trim();
    if normalized_payment_method_id.is_empty() {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "payment_method_id is required",
        ));
    }
    if payment_method.payment_method_id.trim().is_empty() {
        payment_method.payment_method_id = normalized_payment_method_id.to_owned();
    } else if payment_method.payment_method_id != normalized_payment_method_id {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!(
                "payment_method_id mismatch between path {} and body {}",
                normalized_payment_method_id, payment_method.payment_method_id
            ),
        ));
    }

    persist_admin_payment_method(state.store.as_ref(), &payment_method)
        .await
        .map(Json)
        .map_err(admin_commerce_error_response)
}

pub(crate) async fn delete_payment_method_handler(
    _claims: AuthenticatedAdminClaims,
    Path(payment_method_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match delete_admin_payment_method(state.store.as_ref(), &payment_method_id)
        .await
        .map_err(admin_commerce_error_response)?
    {
        true => Ok(StatusCode::NO_CONTENT),
        false => Err(error_response(
            StatusCode::NOT_FOUND,
            format!("payment method {payment_method_id} not found"),
        )),
    }
}

pub(crate) async fn list_payment_method_credential_bindings_handler(
    _claims: AuthenticatedAdminClaims,
    Path(payment_method_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<PaymentMethodCredentialBindingRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_admin_payment_method_credential_bindings(state.store.as_ref(), &payment_method_id)
        .await
        .map(Json)
        .map_err(admin_commerce_error_response)
}

pub(crate) async fn replace_payment_method_credential_bindings_handler(
    _claims: AuthenticatedAdminClaims,
    Path(payment_method_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(mut bindings): Json<Vec<PaymentMethodCredentialBindingRecord>>,
) -> Result<Json<Vec<PaymentMethodCredentialBindingRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let normalized_payment_method_id = payment_method_id.trim();
    if normalized_payment_method_id.is_empty() {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "payment_method_id is required",
        ));
    }
    for binding in &mut bindings {
        if binding.payment_method_id.trim().is_empty() {
            binding.payment_method_id = normalized_payment_method_id.to_owned();
        } else if binding.payment_method_id != normalized_payment_method_id {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "binding {} does not belong to payment method {}",
                    binding.binding_id, normalized_payment_method_id
                ),
            ));
        }
    }

    replace_admin_payment_method_credential_bindings(
        state.store.as_ref(),
        normalized_payment_method_id,
        &bindings,
    )
    .await
    .map(Json)
    .map_err(admin_commerce_error_response)
}

pub(crate) async fn list_commerce_payment_events_handler(
    _claims: AuthenticatedAdminClaims,
    Path(order_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommercePaymentEventRecord>>, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .list_commerce_payment_events_for_order(&order_id)
        .await
        .map(Json)
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load commerce payment events for order {order_id}: {error}"),
            )
        })
}

pub(crate) async fn list_commerce_payment_attempts_handler(
    _claims: AuthenticatedAdminClaims,
    Path(order_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommercePaymentAttemptRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_payment_attempts_for_order(state.store.as_ref(), &order_id)
        .await
        .map(Json)
        .map_err(admin_commerce_error_response)
}

pub(crate) async fn list_commerce_refunds_handler(
    _claims: AuthenticatedAdminClaims,
    Path(order_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommerceRefundRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_admin_commerce_refunds_for_order(state.store.as_ref(), &order_id)
        .await
        .map(Json)
        .map_err(admin_commerce_error_response)
}

pub(crate) async fn create_commerce_refund_handler(
    _claims: AuthenticatedAdminClaims,
    Path(order_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<AdminCommerceRefundCreateRequest>,
) -> Result<Json<CommerceRefundRecord>, (StatusCode, Json<ErrorResponse>)> {
    create_admin_commerce_refund(
        state.store.as_ref(),
        state.commercial_billing.as_deref(),
        &state.secret_manager,
        &order_id,
        &request,
    )
    .await
    .map(Json)
    .map_err(admin_commerce_error_response)
}

pub(crate) async fn list_commerce_webhook_inbox_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommerceWebhookInboxRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_admin_commerce_webhook_inbox(state.store.as_ref())
        .await
        .map(Json)
        .map_err(admin_commerce_error_response)
}

pub(crate) async fn list_commerce_webhook_delivery_attempts_handler(
    _claims: AuthenticatedAdminClaims,
    Path(webhook_inbox_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommerceWebhookDeliveryAttemptRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_admin_commerce_webhook_delivery_attempts(state.store.as_ref(), &webhook_inbox_id)
        .await
        .map(Json)
        .map_err(admin_commerce_error_response)
}

pub(crate) async fn list_commerce_reconciliation_runs_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommerceReconciliationRunRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_admin_commerce_reconciliation_runs(state.store.as_ref())
        .await
        .map(Json)
        .map_err(admin_commerce_error_response)
}

pub(crate) async fn create_commerce_reconciliation_run_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<AdminCommerceReconciliationRunCreateRequest>,
) -> Result<Json<CommerceReconciliationRunRecord>, (StatusCode, Json<ErrorResponse>)> {
    create_admin_commerce_reconciliation_run(state.store.as_ref(), &state.secret_manager, &request)
        .await
        .map(Json)
        .map_err(admin_commerce_error_response)
}

pub(crate) async fn list_commerce_reconciliation_items_handler(
    _claims: AuthenticatedAdminClaims,
    Path(reconciliation_run_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommerceReconciliationItemRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_admin_commerce_reconciliation_items(state.store.as_ref(), &reconciliation_run_id)
        .await
        .map(Json)
        .map_err(admin_commerce_error_response)
}

pub(crate) async fn get_commerce_order_audit_handler(
    _claims: AuthenticatedAdminClaims,
    Path(order_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<CommerceOrderAuditRecord>, (StatusCode, Json<ErrorResponse>)> {
    let order = state
        .store
        .list_commerce_orders()
        .await
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load commerce order {order_id}: {error}"),
            )
        })?
        .into_iter()
        .find(|order| order.order_id == order_id)
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("commerce order {order_id} not found"),
            )
        })?;

    let mut payment_events = state
        .store
        .list_commerce_payment_events_for_order(&order_id)
        .await
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load commerce payment events for order {order_id}: {error}"),
            )
        })?;
    payment_events.sort_by(|left, right| {
        right
            .processed_at_ms
            .unwrap_or(right.received_at_ms)
            .cmp(&left.processed_at_ms.unwrap_or(left.received_at_ms))
            .then_with(|| right.payment_event_id.cmp(&left.payment_event_id))
    });

    let coupon_reservation = match order.coupon_reservation_id.as_deref() {
        Some(coupon_reservation_id) => state
            .store
            .find_coupon_reservation_record(coupon_reservation_id)
            .await
            .map_err(|error| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "failed to load coupon reservation {coupon_reservation_id} for order {order_id}: {error}"
                    ),
                )
            })?,
        None => None,
    };

    let coupon_redemption = match order.coupon_redemption_id.as_deref() {
        Some(coupon_redemption_id) => state
            .store
            .find_coupon_redemption_record(coupon_redemption_id)
            .await
            .map_err(|error| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "failed to load coupon redemption {coupon_redemption_id} for order {order_id}: {error}"
                    ),
                )
            })?,
        None => None,
    };

    let mut coupon_rollbacks = match coupon_redemption.as_ref() {
        Some(redemption) => state
            .store
            .list_coupon_rollback_records()
            .await
            .map_err(|error| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "failed to load coupon rollback evidence for order {order_id}: {error}"
                    ),
                )
            })?
            .into_iter()
            .filter(|rollback| rollback.coupon_redemption_id == redemption.coupon_redemption_id)
            .collect::<Vec<_>>(),
        None => Vec::new(),
    };
    coupon_rollbacks.sort_by(|left, right| {
        right
            .updated_at_ms
            .cmp(&left.updated_at_ms)
            .then_with(|| right.coupon_rollback_id.cmp(&left.coupon_rollback_id))
    });

    let coupon_code = if let Some(redemption) = coupon_redemption.as_ref() {
        state
            .store
            .find_coupon_code_record(&redemption.coupon_code_id)
            .await
            .map_err(|error| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "failed to load coupon code {} for order {order_id}: {error}",
                        redemption.coupon_code_id
                    ),
                )
            })?
    } else if let Some(reservation) = coupon_reservation.as_ref() {
        state
            .store
            .find_coupon_code_record(&reservation.coupon_code_id)
            .await
            .map_err(|error| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "failed to load coupon code {} for order {order_id}: {error}",
                        reservation.coupon_code_id
                    ),
                )
            })?
    } else if let Some(applied_coupon_code) = order.applied_coupon_code.as_deref() {
        state
            .store
            .find_coupon_code_record_by_value(applied_coupon_code)
            .await
            .map_err(|error| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "failed to load coupon code {applied_coupon_code} for order {order_id}: {error}"
                    ),
                )
            })?
    } else {
        None
    };

    let coupon_template_id = coupon_redemption
        .as_ref()
        .map(|redemption| redemption.coupon_template_id.as_str())
        .or_else(|| {
            coupon_code
                .as_ref()
                .map(|code| code.coupon_template_id.as_str())
        });
    let coupon_template = match coupon_template_id {
        Some(coupon_template_id) => state
            .store
            .find_coupon_template_record(coupon_template_id)
            .await
            .map_err(|error| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "failed to load coupon template {coupon_template_id} for order {order_id}: {error}"
                    ),
                )
            })?,
        None => None,
    };

    let marketing_campaign = match order.marketing_campaign_id.as_deref() {
        Some(marketing_campaign_id) => state
            .store
            .list_marketing_campaign_records()
            .await
            .map_err(|error| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "failed to load marketing campaign evidence for order {order_id}: {error}"
                    ),
                )
            })?
            .into_iter()
            .find(|record| record.marketing_campaign_id == marketing_campaign_id),
        None => None,
    };

    Ok(Json(CommerceOrderAuditRecord {
        order,
        payment_events,
        coupon_reservation,
        coupon_redemption,
        coupon_rollbacks,
        coupon_code,
        coupon_template,
        marketing_campaign,
    }))
}
