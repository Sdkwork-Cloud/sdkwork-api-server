use crate::constants::{
    COMMERCE_PAYMENT_CHANNEL_HOSTED_CHECKOUT, COMMERCE_PAYMENT_CHANNEL_OPERATOR_SETTLEMENT,
    COMMERCE_PAYMENT_CHANNEL_SCAN_QR, COMMERCE_PAYMENT_PROVIDER_ALIPAY,
    COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB, COMMERCE_PAYMENT_PROVIDER_NO_PAYMENT_REQUIRED,
    COMMERCE_PAYMENT_PROVIDER_STRIPE, COMMERCE_PAYMENT_PROVIDER_WECHAT_PAY,
};
use crate::error::{CommerceError, CommerceResult};
use crate::types::{
    PortalCommerceCheckoutSession, PortalCommerceCheckoutSessionMethod,
    PortalCommercePaymentEventRequest,
};
use sdkwork_api_domain_commerce::{
    CommerceOrderRecord, PaymentMethodCredentialBindingRecord, PaymentMethodRecord,
};
use sdkwork_api_storage_core::AdminStore;

pub async fn list_admin_payment_methods(
    store: &dyn AdminStore,
) -> CommerceResult<Vec<PaymentMethodRecord>> {
    let mut methods = store
        .list_payment_methods()
        .await
        .map_err(CommerceError::from)?;
    sort_payment_methods(&mut methods);
    Ok(methods)
}

pub async fn persist_admin_payment_method(
    store: &dyn AdminStore,
    payment_method: &PaymentMethodRecord,
) -> CommerceResult<PaymentMethodRecord> {
    validate_payment_method_record(payment_method)?;
    store
        .upsert_payment_method(payment_method)
        .await
        .map_err(CommerceError::from)
}

pub async fn delete_admin_payment_method(
    store: &dyn AdminStore,
    payment_method_id: &str,
) -> CommerceResult<bool> {
    let payment_method_id = payment_method_id.trim();
    if payment_method_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "payment_method_id is required".to_owned(),
        ));
    }
    store
        .delete_payment_method(payment_method_id)
        .await
        .map_err(CommerceError::from)
}

pub async fn list_admin_payment_method_credential_bindings(
    store: &dyn AdminStore,
    payment_method_id: &str,
) -> CommerceResult<Vec<PaymentMethodCredentialBindingRecord>> {
    let payment_method_id = payment_method_id.trim();
    if payment_method_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "payment_method_id is required".to_owned(),
        ));
    }

    let mut bindings = store
        .list_payment_method_credential_bindings(payment_method_id)
        .await
        .map_err(CommerceError::from)?;
    bindings.sort_by(|left, right| {
        left.usage_kind
            .cmp(&right.usage_kind)
            .then_with(|| right.updated_at_ms.cmp(&left.updated_at_ms))
            .then_with(|| left.binding_id.cmp(&right.binding_id))
    });
    Ok(bindings)
}

pub async fn replace_admin_payment_method_credential_bindings(
    store: &dyn AdminStore,
    payment_method_id: &str,
    bindings: &[PaymentMethodCredentialBindingRecord],
) -> CommerceResult<Vec<PaymentMethodCredentialBindingRecord>> {
    let payment_method_id = payment_method_id.trim();
    if payment_method_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "payment_method_id is required".to_owned(),
        ));
    }

    let mut usage_guard = std::collections::BTreeSet::new();
    for binding in bindings {
        validate_payment_method_credential_binding(binding, payment_method_id)?;
        if !usage_guard.insert(binding.usage_kind.to_ascii_lowercase()) {
            return Err(CommerceError::InvalidInput(format!(
                "duplicate usage_kind {} for payment method {}",
                binding.usage_kind, payment_method_id
            )));
        }
    }

    let existing = store
        .list_payment_method_credential_bindings(payment_method_id)
        .await
        .map_err(CommerceError::from)?;

    let retained_usages = bindings
        .iter()
        .map(|binding| binding.usage_kind.to_ascii_lowercase())
        .collect::<std::collections::BTreeSet<_>>();
    for existing_binding in existing {
        if !retained_usages.contains(&existing_binding.usage_kind.to_ascii_lowercase()) {
            store
                .delete_payment_method_credential_binding(
                    payment_method_id,
                    &existing_binding.binding_id,
                )
                .await
                .map_err(CommerceError::from)?;
        }
    }

    for binding in bindings {
        store
            .upsert_payment_method_credential_binding(binding)
            .await
            .map_err(CommerceError::from)?;
    }

    list_admin_payment_method_credential_bindings(store, payment_method_id).await
}

pub async fn list_portal_commerce_payment_methods(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<Vec<PaymentMethodRecord>> {
    let order = crate::load_project_commerce_order(store, user_id, project_id, order_id).await?;
    let mut methods = store
        .list_payment_methods()
        .await
        .map_err(CommerceError::from)?
        .into_iter()
        .filter(|method| supports_order(method, &order))
        .collect::<Vec<_>>();
    sort_payment_methods(&mut methods);
    Ok(methods)
}

pub(crate) async fn resolve_checkout_method_for_payment_event(
    store: &dyn AdminStore,
    order: &CommerceOrderRecord,
    request: &PortalCommercePaymentEventRequest,
) -> CommerceResult<Option<PortalCommerceCheckoutSessionMethod>> {
    let Some(checkout_method_id) = request
        .checkout_method_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };

    load_supported_checkout_methods(store, order)
        .await?
        .into_iter()
        .find(|candidate| candidate.id == checkout_method_id)
        .map(Some)
        .ok_or_else(|| {
            CommerceError::InvalidInput(format!(
                "checkout_method_id {checkout_method_id} is not available for order {}",
                order.order_id
            ))
        })
}

pub(crate) async fn build_checkout_session(
    store: &dyn AdminStore,
    order: &CommerceOrderRecord,
    payment_simulation_enabled: bool,
) -> CommerceResult<PortalCommerceCheckoutSession> {
    let reference = format!("PAY-{}", normalize_payment_reference(&order.order_id));
    let guidance = match (order.target_kind.as_str(), order.status.as_str()) {
        ("subscription_plan", "pending_payment") => {
            "Settle this checkout to activate the workspace membership and included monthly units."
        }
        ("recharge_pack", "pending_payment") => {
            "Settle this checkout to apply the recharge pack and restore workspace quota headroom."
        }
        ("custom_recharge", "pending_payment") => {
            "Settle this checkout to apply the custom recharge amount and restore workspace quota headroom."
        }
        ("coupon_redemption", "fulfilled") => {
            "This order required no external payment and was fulfilled immediately at redemption time."
        }
        (_, "fulfilled") => {
            "This checkout session is closed because the order has already been settled."
        }
        (_, "canceled") => {
            "This checkout session is closed because the order was canceled before settlement."
        }
        (_, "failed") => "This checkout session is closed because the payment flow failed.",
        (_, "refunded") => {
            "This checkout session is closed because the order was refunded and quota side effects were rolled back."
        }
        _ => {
            "This checkout session describes how the current order can move through the payment rail."
        }
    };
    let configured_methods = load_supported_checkout_methods(store, order).await?;

    let (session_status, provider, mode, methods) = match order.status.as_str() {
        "pending_payment" => (
            "open",
            COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB,
            COMMERCE_PAYMENT_CHANNEL_OPERATOR_SETTLEMENT,
            configured_methods,
        ),
        "fulfilled"
            if order.target_kind == "coupon_redemption" || order.payable_price_cents == 0 =>
        {
            (
                "not_required",
                COMMERCE_PAYMENT_PROVIDER_NO_PAYMENT_REQUIRED,
                "instant_fulfillment",
                Vec::new(),
            )
        }
        "fulfilled" => (
            "settled",
            COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB,
            "closed",
            Vec::new(),
        ),
        "canceled" => (
            "canceled",
            COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB,
            "closed",
            Vec::new(),
        ),
        "failed" => (
            "failed",
            COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB,
            "closed",
            Vec::new(),
        ),
        "refunded" => (
            "refunded",
            COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB,
            "closed",
            Vec::new(),
        ),
        _ => (
            "closed",
            COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB,
            "closed",
            Vec::new(),
        ),
    };

    Ok(PortalCommerceCheckoutSession {
        order_id: order.order_id.clone(),
        order_status: order.status.clone(),
        session_status: session_status.to_owned(),
        provider: provider.to_owned(),
        mode: mode.to_owned(),
        reference,
        payable_price_label: order.payable_price_label.clone(),
        guidance: guidance.to_owned(),
        payment_simulation_enabled,
        methods,
    })
}

async fn load_supported_checkout_methods(
    store: &dyn AdminStore,
    order: &CommerceOrderRecord,
) -> CommerceResult<Vec<PortalCommerceCheckoutSessionMethod>> {
    let mut methods = base_checkout_methods(order);

    if order.payable_price_cents == 0 {
        return Ok(methods);
    }

    let mut configured_methods = store
        .list_payment_methods()
        .await
        .map_err(CommerceError::from)?
        .into_iter()
        .filter(|method| supports_order(method, order))
        .map(|method| map_payment_method_record_to_checkout_method(order, &method))
        .collect::<Vec<_>>();

    if configured_methods.is_empty() {
        configured_methods = legacy_planned_checkout_methods(order);
    }

    methods.extend(configured_methods);
    Ok(methods)
}

fn base_checkout_methods(order: &CommerceOrderRecord) -> Vec<PortalCommerceCheckoutSessionMethod> {
    vec![
        build_checkout_session_method(
            order,
            "manual_settlement",
            "Manual settlement",
            "Use the portal settlement action in desktop or lab mode to finalize the order.",
            "settle_order",
            "available",
            COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB,
            COMMERCE_PAYMENT_CHANNEL_OPERATOR_SETTLEMENT,
            "operator_action",
            "MANUAL",
            None,
            "manual",
            true,
            false,
            false,
            false,
        ),
        build_checkout_session_method(
            order,
            "cancel_order",
            "Cancel checkout",
            "Close the pending order without applying quota or membership side effects.",
            "cancel_order",
            "available",
            COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB,
            COMMERCE_PAYMENT_CHANNEL_OPERATOR_SETTLEMENT,
            "operator_action",
            "CANCEL",
            None,
            "manual",
            false,
            false,
            false,
            false,
        ),
    ]
}

fn legacy_planned_checkout_methods(
    order: &CommerceOrderRecord,
) -> Vec<PortalCommerceCheckoutSessionMethod> {
    vec![
        build_checkout_session_method(
            order,
            "stripe_checkout",
            "Stripe checkout",
            "Hosted card and wallet checkout rail for global business payments, subscriptions, and webhook-driven settlement.",
            "provider_handoff",
            "planned",
            COMMERCE_PAYMENT_PROVIDER_STRIPE,
            COMMERCE_PAYMENT_CHANNEL_HOSTED_CHECKOUT,
            "hosted_checkout",
            "STRIPE",
            None,
            "stripe_signature",
            true,
            true,
            true,
            true,
        ),
        build_checkout_session_method(
            order,
            "alipay_qr",
            "Alipay QR",
            "Mainland China scan-to-pay rail for consumer and enterprise Alipay settlement with callback confirmation.",
            "provider_handoff",
            "planned",
            COMMERCE_PAYMENT_PROVIDER_ALIPAY,
            COMMERCE_PAYMENT_CHANNEL_SCAN_QR,
            "qr_code",
            "ALIPAY",
            Some(build_checkout_qr_payload("alipay_qr", &order.order_id)),
            "alipay_rsa_sha256",
            true,
            false,
            false,
            true,
        ),
        build_checkout_session_method(
            order,
            "wechat_pay_qr",
            "WeChat Pay QR",
            "Native WeChat Pay scan rail for real-time QR settlement, webhook confirmation, and refund lifecycle callbacks.",
            "provider_handoff",
            "planned",
            COMMERCE_PAYMENT_PROVIDER_WECHAT_PAY,
            COMMERCE_PAYMENT_CHANNEL_SCAN_QR,
            "qr_code",
            "WECHAT",
            Some(build_checkout_qr_payload("wechat_pay_qr", &order.order_id)),
            "wechatpay_rsa_sha256",
            true,
            false,
            false,
            true,
        ),
    ]
}

fn supports_order(method: &PaymentMethodRecord, order: &CommerceOrderRecord) -> bool {
    method.enabled
        && (method.supported_currency_codes.is_empty()
            || method
                .supported_currency_codes
                .iter()
                .any(|currency| currency.eq_ignore_ascii_case(&order.currency_code)))
        && (method.supported_order_kinds.is_empty()
            || method
                .supported_order_kinds
                .iter()
                .any(|kind| kind.eq_ignore_ascii_case(&order.target_kind)))
}

fn map_payment_method_record_to_checkout_method(
    order: &CommerceOrderRecord,
    method: &PaymentMethodRecord,
) -> PortalCommerceCheckoutSessionMethod {
    let action = match method.channel.as_str() {
        COMMERCE_PAYMENT_CHANNEL_OPERATOR_SETTLEMENT => "settle_order",
        _ => "provider_handoff",
    };
    let availability = if method.enabled {
        "available"
    } else {
        "disabled"
    };
    let session_kind = match method.channel.as_str() {
        COMMERCE_PAYMENT_CHANNEL_HOSTED_CHECKOUT => "hosted_checkout",
        COMMERCE_PAYMENT_CHANNEL_SCAN_QR => "qr_code",
        COMMERCE_PAYMENT_CHANNEL_OPERATOR_SETTLEMENT => "operator_action",
        _ => "provider_handoff",
    };
    let session_reference_prefix = method.provider.to_ascii_uppercase();
    let qr_code_payload = if method.channel == COMMERCE_PAYMENT_CHANNEL_SCAN_QR {
        Some(build_checkout_qr_payload(
            &method.payment_method_id,
            &order.order_id,
        ))
    } else {
        None
    };
    let supports_refund = supports_capability(method, "refund");
    let supports_partial_refund = supports_capability(method, "partial_refund");
    let recommended = supports_capability(method, "recommended");
    let supports_webhook = method.callback_strategy.contains("webhook");

    build_checkout_session_method(
        order,
        &method.payment_method_id,
        &method.display_name,
        &method.description,
        action,
        availability,
        &method.provider,
        &method.channel,
        session_kind,
        &session_reference_prefix,
        qr_code_payload,
        &method.callback_strategy,
        supports_refund,
        supports_partial_refund,
        recommended,
        supports_webhook,
    )
}

fn supports_capability(method: &PaymentMethodRecord, code: &str) -> bool {
    method
        .capability_codes
        .iter()
        .any(|capability| capability.eq_ignore_ascii_case(code))
}

fn sort_payment_methods(methods: &mut [PaymentMethodRecord]) {
    methods.sort_by(|left, right| {
        right
            .enabled
            .cmp(&left.enabled)
            .then_with(|| left.sort_order.cmp(&right.sort_order))
            .then_with(|| right.updated_at_ms.cmp(&left.updated_at_ms))
            .then_with(|| left.payment_method_id.cmp(&right.payment_method_id))
    });
}

#[allow(clippy::too_many_arguments)]
fn build_checkout_session_method(
    order: &CommerceOrderRecord,
    id: &str,
    label: &str,
    detail: &str,
    action: &str,
    availability: &str,
    provider: &str,
    channel: &str,
    session_kind: &str,
    session_reference_prefix: &str,
    qr_code_payload: Option<String>,
    webhook_verification: &str,
    supports_refund: bool,
    supports_partial_refund: bool,
    recommended: bool,
    supports_webhook: bool,
) -> PortalCommerceCheckoutSessionMethod {
    PortalCommerceCheckoutSessionMethod {
        id: id.to_owned(),
        label: label.to_owned(),
        detail: detail.to_owned(),
        action: action.to_owned(),
        availability: availability.to_owned(),
        provider: provider.to_owned(),
        channel: channel.to_owned(),
        session_kind: session_kind.to_owned(),
        session_reference: build_checkout_session_reference(
            session_reference_prefix,
            &order.order_id,
        ),
        qr_code_payload,
        webhook_verification: webhook_verification.to_owned(),
        supports_refund,
        supports_partial_refund,
        recommended,
        supports_webhook,
    }
}

fn build_checkout_session_reference(prefix: &str, order_id: &str) -> String {
    format!("{prefix}-{}", normalize_payment_reference(order_id))
}

fn build_checkout_qr_payload(method_id: &str, order_id: &str) -> String {
    format!(
        "sdkworkpay://{method_id}/{}",
        normalize_payment_reference(order_id)
    )
}

fn normalize_payment_reference(order_id: &str) -> String {
    order_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_uppercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn validate_payment_method_record(payment_method: &PaymentMethodRecord) -> CommerceResult<()> {
    if payment_method.payment_method_id.trim().is_empty() {
        return Err(CommerceError::InvalidInput(
            "payment_method_id is required".to_owned(),
        ));
    }
    if payment_method.display_name.trim().is_empty() {
        return Err(CommerceError::InvalidInput(
            "display_name is required".to_owned(),
        ));
    }
    if payment_method.provider.trim().is_empty() {
        return Err(CommerceError::InvalidInput(
            "provider is required".to_owned(),
        ));
    }
    if payment_method.channel.trim().is_empty() {
        return Err(CommerceError::InvalidInput(
            "channel is required".to_owned(),
        ));
    }
    if payment_method.callback_strategy.trim().is_empty() {
        return Err(CommerceError::InvalidInput(
            "callback_strategy is required".to_owned(),
        ));
    }
    Ok(())
}

fn validate_payment_method_credential_binding(
    binding: &PaymentMethodCredentialBindingRecord,
    payment_method_id: &str,
) -> CommerceResult<()> {
    if binding.binding_id.trim().is_empty() {
        return Err(CommerceError::InvalidInput(
            "binding_id is required".to_owned(),
        ));
    }
    if binding.payment_method_id.trim() != payment_method_id {
        return Err(CommerceError::InvalidInput(format!(
            "binding {} does not belong to payment method {}",
            binding.binding_id, payment_method_id
        )));
    }
    if binding.usage_kind.trim().is_empty() {
        return Err(CommerceError::InvalidInput(
            "usage_kind is required".to_owned(),
        ));
    }
    if binding.credential_tenant_id.trim().is_empty() {
        return Err(CommerceError::InvalidInput(
            "credential_tenant_id is required".to_owned(),
        ));
    }
    if binding.credential_provider_id.trim().is_empty() {
        return Err(CommerceError::InvalidInput(
            "credential_provider_id is required".to_owned(),
        ));
    }
    if binding.credential_key_reference.trim().is_empty() {
        return Err(CommerceError::InvalidInput(
            "credential_key_reference is required".to_owned(),
        ));
    }
    Ok(())
}
