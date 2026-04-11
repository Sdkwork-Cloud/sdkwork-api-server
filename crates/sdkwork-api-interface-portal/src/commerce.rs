use super::*;
use sdkwork_api_app_commerce::project_portal_commerce_order_catalog_binding;

#[derive(Debug, Serialize)]
pub(crate) struct PortalCommerceOrderView {
    #[serde(flatten)]
    pub(crate) order: PortalCommerceOrderRecord,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) product_kind: Option<String>,
    pub(crate) transaction_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) product_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) offer_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) publication_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) publication_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) publication_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) publication_revision_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) publication_version: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) publication_source_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) publication_effective_from_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) pricing_rate_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) pricing_metric_code: Option<String>,
}

impl From<PortalCommerceOrderRecord> for PortalCommerceOrderView {
    fn from(order: PortalCommerceOrderRecord) -> Self {
        let binding = project_portal_commerce_order_catalog_binding(&order);
        Self {
            product_kind: portal_commerce_product_kind(&order.target_kind).map(str::to_owned),
            transaction_kind: portal_commerce_transaction_kind(&order.target_kind).to_owned(),
            product_id: binding.product_id,
            offer_id: binding.offer_id,
            publication_id: binding.publication_id,
            publication_kind: binding.publication_kind,
            publication_status: binding.publication_status,
            publication_revision_id: binding.publication_revision_id,
            publication_version: binding.publication_version,
            publication_source_kind: binding.publication_source_kind,
            publication_effective_from_ms: binding.publication_effective_from_ms,
            pricing_rate_id: binding.pricing_rate_id,
            pricing_metric_code: binding.pricing_metric_code,
            order,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct PortalOrderCenterEntry {
    pub(crate) order: PortalCommerceOrderView,
    pub(crate) payment_events: Vec<PortalCommercePaymentEventRecord>,
    pub(crate) latest_payment_event: Option<PortalCommercePaymentEventRecord>,
    pub(crate) checkout_session: PortalCommerceCheckoutSession,
}

#[derive(Debug, Serialize)]
pub(crate) struct PortalCommerceReconciliationSummary {
    pub(crate) account_id: u64,
    pub(crate) last_reconciled_order_id: String,
    pub(crate) last_reconciled_order_updated_at_ms: u64,
    pub(crate) last_reconciled_order_created_at_ms: u64,
    pub(crate) last_reconciled_at_ms: u64,
    pub(crate) backlog_order_count: usize,
    pub(crate) checkpoint_lag_ms: u64,
    pub(crate) healthy: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct PortalCommerceOrderCenterResponse {
    project_id: String,
    payment_simulation_enabled: bool,
    membership: Option<PortalProjectMembershipRecord>,
    reconciliation: Option<PortalCommerceReconciliationSummary>,
    orders: Vec<PortalOrderCenterEntry>,
}

pub(crate) async fn commerce_catalog_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalCommerceCatalog>, StatusCode> {
    let _workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_portal_commerce_catalog(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn commerce_quote_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<PortalCommerceQuoteRequest>,
) -> Result<Json<PortalCommerceQuote>, (StatusCode, Json<ErrorResponse>)> {
    let _workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    preview_portal_commerce_quote(state.store.as_ref(), &request)
        .await
        .map(Json)
        .map_err(portal_commerce_error_response)
}

pub(crate) async fn list_commerce_orders_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<PortalCommerceOrderView>>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    list_project_commerce_orders(state.store.as_ref(), &workspace.project.id)
        .await
        .map(|orders| {
            orders
                .into_iter()
                .map(PortalCommerceOrderView::from)
                .collect::<Vec<_>>()
        })
        .map(Json)
        .map_err(portal_commerce_error_response)
}

pub(crate) async fn get_commerce_order_handler(
    claims: AuthenticatedPortalClaims,
    Path(order_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalCommerceOrderView>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    load_portal_commerce_order(
        state.store.as_ref(),
        &claims.claims().sub,
        &workspace.project.id,
        &order_id,
    )
    .await
    .map(PortalCommerceOrderView::from)
    .map(Json)
    .map_err(portal_commerce_error_response)
}

pub(crate) async fn commerce_order_center_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalCommerceOrderCenterResponse>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    let orders = list_project_commerce_orders(state.store.as_ref(), &workspace.project.id)
        .await
        .map_err(portal_commerce_error_response)?;
    let membership = load_project_membership(state.store.as_ref(), &workspace.project.id)
        .await
        .map_err(portal_commerce_error_response)?;

    let mut order_center_entries = Vec::with_capacity(orders.len());
    for order in orders {
        let mut payment_events = state
            .store
            .list_commerce_payment_events_for_order(&order.order_id)
            .await
            .map_err(CommerceError::from)
            .map_err(portal_commerce_error_response)?;
        payment_events.sort_by(|left, right| {
            right
                .received_at_ms
                .cmp(&left.received_at_ms)
                .then_with(|| right.payment_event_id.cmp(&left.payment_event_id))
        });
        let latest_payment_event = payment_events.first().cloned();
        let checkout_session = load_portal_commerce_checkout_session_with_policy(
            state.store.as_ref(),
            &claims.claims().sub,
            &workspace.project.id,
            &order.order_id,
            state.payment_simulation_enabled,
        )
        .await
        .map_err(portal_commerce_error_response)?;
        order_center_entries.push(PortalOrderCenterEntry {
            order: PortalCommerceOrderView::from(order),
            payment_events,
            latest_payment_event,
            checkout_session,
        });
    }
    let reconciliation =
        load_portal_commerce_reconciliation_summary(&state, &workspace, &order_center_entries)
            .await?;

    Ok(Json(PortalCommerceOrderCenterResponse {
        project_id: workspace.project.id,
        payment_simulation_enabled: state.payment_simulation_enabled,
        membership,
        reconciliation,
        orders: order_center_entries,
    }))
}

pub(crate) async fn create_commerce_order_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<PortalCommerceQuoteRequest>,
) -> Result<(StatusCode, Json<PortalCommerceOrderView>), (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    submit_portal_commerce_order(
        state.store.as_ref(),
        &claims.claims().sub,
        &workspace.project.id,
        &request,
    )
    .await
    .map(|order| {
        (
            StatusCode::CREATED,
            Json(PortalCommerceOrderView::from(order)),
        )
    })
    .map_err(portal_commerce_error_response)
}

pub(crate) async fn settle_commerce_order_handler(
    claims: AuthenticatedPortalClaims,
    Path(order_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalCommerceOrderView>, (StatusCode, Json<ErrorResponse>)> {
    assert_payment_simulation_enabled(&state)?;

    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    settle_portal_commerce_order_with_billing(
        state.store.as_ref(),
        state.commercial_billing.as_deref(),
        &claims.claims().sub,
        &workspace.project.id,
        &order_id,
    )
    .await
    .map(PortalCommerceOrderView::from)
    .map(Json)
    .map_err(portal_commerce_error_response)
}

pub(crate) async fn cancel_commerce_order_handler(
    claims: AuthenticatedPortalClaims,
    Path(order_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalCommerceOrderView>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    cancel_portal_commerce_order(
        state.store.as_ref(),
        &claims.claims().sub,
        &workspace.project.id,
        &order_id,
    )
    .await
    .map(PortalCommerceOrderView::from)
    .map(Json)
    .map_err(portal_commerce_error_response)
}

pub(crate) async fn apply_commerce_payment_event_handler(
    claims: AuthenticatedPortalClaims,
    Path(order_id): Path<String>,
    State(state): State<PortalApiState>,
    Json(request): Json<PortalCommercePaymentEventRequest>,
) -> Result<Json<PortalCommerceOrderView>, (StatusCode, Json<ErrorResponse>)> {
    assert_payment_simulation_enabled(&state)?;

    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    apply_portal_commerce_payment_event_with_billing(
        state.store.as_ref(),
        state.commercial_billing.as_deref(),
        &claims.claims().sub,
        &workspace.project.id,
        &order_id,
        &request,
    )
    .await
    .map(PortalCommerceOrderView::from)
    .map(Json)
    .map_err(portal_commerce_error_response)
}

pub(crate) async fn create_commerce_payment_attempt_handler(
    claims: AuthenticatedPortalClaims,
    Path(order_id): Path<String>,
    State(state): State<PortalApiState>,
    Json(request): Json<PortalCommercePaymentAttemptCreateRequest>,
) -> Result<(StatusCode, Json<CommercePaymentAttemptRecord>), (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    create_portal_commerce_payment_attempt(
        state.store.as_ref(),
        &state.secret_manager,
        &claims.claims().sub,
        &workspace.project.id,
        &order_id,
        &request,
    )
    .await
    .map(|payment_attempt| (StatusCode::CREATED, Json(payment_attempt)))
    .map_err(portal_commerce_error_response)
}

pub(crate) async fn list_commerce_payment_attempts_handler(
    claims: AuthenticatedPortalClaims,
    Path(order_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<CommercePaymentAttemptRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    list_portal_commerce_payment_attempts(
        state.store.as_ref(),
        &claims.claims().sub,
        &workspace.project.id,
        &order_id,
    )
    .await
    .map(Json)
    .map_err(portal_commerce_error_response)
}

pub(crate) async fn list_commerce_payment_methods_handler(
    claims: AuthenticatedPortalClaims,
    Path(order_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<PaymentMethodRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    list_portal_commerce_payment_methods(
        state.store.as_ref(),
        &claims.claims().sub,
        &workspace.project.id,
        &order_id,
    )
    .await
    .map(Json)
    .map_err(portal_commerce_error_response)
}

pub(crate) async fn get_commerce_checkout_session_handler(
    claims: AuthenticatedPortalClaims,
    Path(order_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalCommerceCheckoutSession>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    load_portal_commerce_checkout_session_with_policy(
        state.store.as_ref(),
        &claims.claims().sub,
        &workspace.project.id,
        &order_id,
        state.payment_simulation_enabled,
    )
    .await
    .map(Json)
    .map_err(portal_commerce_error_response)
}

pub(crate) async fn get_commerce_payment_attempt_handler(
    claims: AuthenticatedPortalClaims,
    Path(payment_attempt_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<Json<CommercePaymentAttemptRecord>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    load_portal_commerce_payment_attempt(
        state.store.as_ref(),
        &claims.claims().sub,
        &workspace.project.id,
        &payment_attempt_id,
    )
    .await
    .map(Json)
    .map_err(portal_commerce_error_response)
}

pub(crate) async fn stripe_webhook_handler(
    Path(payment_method_id): Path<String>,
    State(state): State<PortalApiState>,
    headers: HeaderMap,
    payload: Bytes,
) -> Result<Json<PortalCommerceWebhookAck>, (StatusCode, Json<ErrorResponse>)> {
    let payload = std::str::from_utf8(payload.as_ref()).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorBody {
                    message: "webhook payload must be valid utf-8".to_owned(),
                },
            }),
        )
    })?;

    process_portal_stripe_webhook(
        state.store.as_ref(),
        state.commercial_billing.as_deref(),
        &state.secret_manager,
        &payment_method_id,
        headers
            .get("Stripe-Signature")
            .and_then(|value| value.to_str().ok()),
        payload,
    )
    .await
    .map(Json)
    .map_err(portal_commerce_error_response)
}

pub(crate) async fn get_project_membership_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Option<PortalProjectMembershipRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    load_project_membership(state.store.as_ref(), &workspace.project.id)
        .await
        .map(Json)
        .map_err(portal_commerce_error_response)
}

fn portal_commerce_error_response(error: CommerceError) -> (StatusCode, Json<ErrorResponse>) {
    let status = match error {
        CommerceError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        CommerceError::NotFound(_) => StatusCode::NOT_FOUND,
        CommerceError::Conflict(_) => StatusCode::CONFLICT,
        CommerceError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let body = ErrorResponse {
        error: ErrorBody {
            message: error.to_string(),
        },
    };
    (status, Json(body))
}

fn assert_payment_simulation_enabled(
    state: &PortalApiState,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if state.payment_simulation_enabled {
        return Ok(());
    }

    Err((
        StatusCode::CONFLICT,
        Json(ErrorResponse {
            error: ErrorBody {
                message:
                    "portal payment simulation is disabled; use payment attempts and provider callbacks"
                        .to_owned(),
            },
        }),
    ))
}
