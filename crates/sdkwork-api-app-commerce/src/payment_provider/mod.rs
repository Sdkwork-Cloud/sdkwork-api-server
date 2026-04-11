pub(crate) mod stripe;

use crate::error::{CommerceError, CommerceResult};
use sdkwork_api_app_credential::{resolve_credential_secret_with_manager, CredentialSecretManager};
use sdkwork_api_domain_commerce::{
    CommerceOrderRecord, PaymentMethodCredentialBindingRecord, PaymentMethodRecord,
};
use sdkwork_api_storage_core::AdminStore;
use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub(crate) struct PaymentMethodCheckoutConfig {
    pub success_url: Option<String>,
    pub cancel_url: Option<String>,
    pub product_name: Option<String>,
    pub payment_method_types: Vec<String>,
    pub expires_in_minutes: Option<u64>,
    pub customer_creation: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub(crate) struct PaymentMethodConfig {
    pub checkout: PaymentMethodCheckoutConfig,
}

#[derive(Debug, Clone)]
pub(crate) struct PaymentMethodSecretBundle {
    pub api_secret: String,
    pub webhook_secret: Option<String>,
    pub reconciliation_secret: Option<String>,
}

pub(crate) async fn load_payment_method(
    store: &dyn AdminStore,
    payment_method_id: &str,
) -> CommerceResult<PaymentMethodRecord> {
    let payment_method_id = payment_method_id.trim();
    if payment_method_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "payment_method_id is required".to_owned(),
        ));
    }

    store
        .find_payment_method(payment_method_id)
        .await
        .map_err(CommerceError::from)?
        .ok_or_else(|| {
            CommerceError::NotFound(format!("payment method {payment_method_id} not found"))
        })
}

pub(crate) fn ensure_payment_method_supports_order(
    method: &PaymentMethodRecord,
    order: &CommerceOrderRecord,
    requested_country_code: Option<&str>,
) -> CommerceResult<()> {
    if !method.enabled {
        return Err(CommerceError::Conflict(format!(
            "payment method {} is disabled",
            method.payment_method_id
        )));
    }
    if !method.supported_currency_codes.is_empty()
        && !method
            .supported_currency_codes
            .iter()
            .any(|currency| currency.eq_ignore_ascii_case(&order.currency_code))
    {
        return Err(CommerceError::InvalidInput(format!(
            "payment method {} does not support currency {}",
            method.payment_method_id, order.currency_code
        )));
    }
    if !method.supported_order_kinds.is_empty()
        && !method
            .supported_order_kinds
            .iter()
            .any(|kind| kind.eq_ignore_ascii_case(&order.target_kind))
    {
        return Err(CommerceError::InvalidInput(format!(
            "payment method {} does not support order kind {}",
            method.payment_method_id, order.target_kind
        )));
    }

    if let Some(country_code) = requested_country_code
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if !method.supported_country_codes.is_empty()
            && !method
                .supported_country_codes
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(country_code))
        {
            return Err(CommerceError::InvalidInput(format!(
                "payment method {} does not support country {}",
                method.payment_method_id, country_code
            )));
        }
    }

    Ok(())
}

pub(crate) fn parse_payment_method_config(
    payment_method: &PaymentMethodRecord,
) -> CommerceResult<PaymentMethodConfig> {
    if payment_method.config_json.trim().is_empty() {
        return Ok(PaymentMethodConfig::default());
    }

    serde_json::from_str::<PaymentMethodConfig>(&payment_method.config_json).map_err(|error| {
        CommerceError::InvalidInput(format!(
            "payment method {} has invalid config_json: {error}",
            payment_method.payment_method_id
        ))
    })
}

pub(crate) async fn resolve_payment_method_secret_bundle(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    payment_method_id: &str,
) -> CommerceResult<PaymentMethodSecretBundle> {
    let bindings = store
        .list_payment_method_credential_bindings(payment_method_id)
        .await
        .map_err(CommerceError::from)?;

    let api_secret = resolve_binding_secret(store, secret_manager, &bindings, "api_secret").await?;
    let webhook_secret =
        resolve_binding_secret_optional(store, secret_manager, &bindings, "webhook_secret").await?;
    let reconciliation_secret =
        resolve_binding_secret_optional(store, secret_manager, &bindings, "reconciliation_secret")
            .await?;

    Ok(PaymentMethodSecretBundle {
        api_secret,
        webhook_secret,
        reconciliation_secret,
    })
}

pub(crate) fn render_template(
    template: &str,
    order: &CommerceOrderRecord,
    payment_attempt_id: &str,
) -> String {
    template
        .replace("{order_id}", &order.order_id)
        .replace("{project_id}", &order.project_id)
        .replace("{user_id}", &order.user_id)
        .replace("{target_kind}", &order.target_kind)
        .replace("{target_id}", &order.target_id)
        .replace("{payment_attempt_id}", payment_attempt_id)
}

async fn resolve_binding_secret(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    bindings: &[PaymentMethodCredentialBindingRecord],
    usage_kind: &str,
) -> CommerceResult<String> {
    let binding = bindings
        .iter()
        .find(|binding| binding.usage_kind.eq_ignore_ascii_case(usage_kind))
        .ok_or_else(|| {
            CommerceError::InvalidInput(format!(
                "credential binding for usage_kind {usage_kind} is required"
            ))
        })?;

    resolve_credential_secret_with_manager(
        store,
        secret_manager,
        &binding.credential_tenant_id,
        &binding.credential_provider_id,
        &binding.credential_key_reference,
    )
    .await
    .map_err(CommerceError::from)
}

async fn resolve_binding_secret_optional(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    bindings: &[PaymentMethodCredentialBindingRecord],
    usage_kind: &str,
) -> CommerceResult<Option<String>> {
    let Some(binding) = bindings
        .iter()
        .find(|binding| binding.usage_kind.eq_ignore_ascii_case(usage_kind))
    else {
        return Ok(None);
    };

    resolve_credential_secret_with_manager(
        store,
        secret_manager,
        &binding.credential_tenant_id,
        &binding.credential_provider_id,
        &binding.credential_key_reference,
    )
    .await
    .map(Some)
    .map_err(CommerceError::from)
}
