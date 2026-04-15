#![allow(dead_code)]

use super::*;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "SDKWORK Admin API",
        version = env!("CARGO_PKG_VERSION"),
        description = "OpenAPI 3.1 schema generated directly from the current admin router implementation."
    ),
    modifiers(&AdminApiDocModifier),
    paths(
        system_paths::health,
        auth_paths::auth_login,
        auth_paths::auth_change_password,
        tenant_paths::tenants_list,
        tenant_paths::tenants_create,
        catalog_paths::tenant_provider_readiness_list,
        tenant_paths::projects_list,
        tenant_paths::projects_create,
        catalog_paths::providers_list,
        catalog_paths::providers_create,
        user_paths::operator_users_list,
        user_paths::operator_users_upsert,
        user_paths::operator_user_status_update,
        user_paths::portal_users_list,
        user_paths::portal_users_upsert,
        user_paths::portal_user_status_update,
        marketing_template_paths::marketing_coupon_templates_list,
        marketing_template_paths::marketing_coupon_templates_create,
        marketing_template_paths::marketing_coupon_templates_status_update,
        marketing_template_paths::marketing_coupon_templates_clone,
        marketing_template_paths::marketing_coupon_templates_compare,
        marketing_template_paths::marketing_coupon_templates_submit_for_approval,
        marketing_template_paths::marketing_coupon_templates_approve,
        marketing_template_paths::marketing_coupon_templates_reject,
        marketing_template_paths::marketing_coupon_templates_publish,
        marketing_template_paths::marketing_coupon_templates_schedule,
        marketing_template_paths::marketing_coupon_templates_retire,
        marketing_template_paths::marketing_coupon_template_lifecycle_audits_list,
        marketing_campaign_paths::marketing_campaigns_list,
        marketing_campaign_paths::marketing_campaigns_create,
        marketing_campaign_paths::marketing_campaigns_status_update,
        marketing_campaign_paths::marketing_campaigns_clone,
        marketing_campaign_paths::marketing_campaigns_compare,
        marketing_campaign_paths::marketing_campaigns_submit_for_approval,
        marketing_campaign_paths::marketing_campaigns_approve,
        marketing_campaign_paths::marketing_campaigns_reject,
        marketing_campaign_paths::marketing_campaigns_publish,
        marketing_campaign_paths::marketing_campaigns_schedule,
        marketing_campaign_paths::marketing_campaigns_retire,
        marketing_campaign_paths::marketing_campaign_lifecycle_audits_list,
        marketing_budget_paths::marketing_budgets_list,
        marketing_budget_paths::marketing_budgets_create,
        marketing_budget_paths::marketing_budgets_status_update,
        marketing_budget_paths::marketing_budgets_activate,
        marketing_budget_paths::marketing_budgets_close,
        marketing_budget_paths::marketing_budget_lifecycle_audits_list,
        marketing_code_paths::marketing_codes_list,
        marketing_code_paths::marketing_codes_create,
        marketing_code_paths::marketing_codes_status_update,
        marketing_code_paths::marketing_codes_disable,
        marketing_code_paths::marketing_codes_restore,
        marketing_code_paths::marketing_code_lifecycle_audits_list,
        marketing_runtime_paths::marketing_reservations_list,
        marketing_runtime_paths::marketing_redemptions_list,
        marketing_runtime_paths::marketing_rollbacks_list,
        gateway_paths::api_keys_list,
        gateway_paths::api_keys_create,
        gateway_paths::api_key_update,
        gateway_paths::api_key_groups_list,
        gateway_paths::api_key_groups_create,
        gateway_paths::api_key_group_update,
        billing_paths::billing_ledger_list,
        billing_paths::billing_events_list,
        billing_paths::billing_events_summary,
        billing_paths::billing_summary,
        billing_paths::billing_pricing_lifecycle_synchronize,
        billing_paths::billing_account_ledger,
        commerce_paths::commerce_orders_recent,
        commerce_paths::commerce_catalog_publications_list,
        commerce_paths::commerce_catalog_publication_detail,
        commerce_paths::commerce_catalog_publication_publish,
        commerce_paths::commerce_catalog_publication_schedule,
        commerce_paths::commerce_catalog_publication_retire,
        commerce_paths::commerce_payment_methods_list,
        commerce_paths::commerce_payment_method_put,
        commerce_paths::commerce_payment_method_delete,
        commerce_paths::commerce_payment_method_bindings_list,
        commerce_paths::commerce_payment_method_bindings_replace,
        commerce_paths::commerce_order_payment_events,
        commerce_paths::commerce_order_payment_attempts,
        commerce_paths::commerce_order_refunds_list,
        commerce_paths::commerce_order_refunds_create,
        commerce_paths::commerce_order_audit,
        commerce_paths::commerce_webhook_inbox_list,
        commerce_paths::commerce_webhook_delivery_attempts_list,
        commerce_paths::commerce_reconciliation_runs_list,
        commerce_paths::commerce_reconciliation_runs_create,
        commerce_paths::commerce_reconciliation_items_list
    ),
    tags(
        (name = "system", description = "Admin health and system-facing routes."),
        (name = "auth", description = "Admin authentication and session management routes."),
        (name = "catalog", description = "Provider and model catalog administration routes."),
        (name = "marketing", description = "Coupon template, campaign, budget, and redemption administration routes."),
        (name = "tenants", description = "Tenant and project administration routes."),
        (name = "users", description = "Operator and portal user administration routes."),
        (name = "gateway", description = "Gateway API key and API key group administration routes."),
        (name = "billing", description = "Billing summary, event, and ledger administration routes."),
        (name = "commerce", description = "Recent order and payment callback audit routes.")
    )
)]
struct AdminApiDoc;

struct AdminApiDocModifier;

impl Modify for AdminApiDocModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.servers = Some(vec![Server::new("/")]);
        openapi
            .components
            .get_or_insert_with(utoipa::openapi::Components::new)
            .add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
    }
}

#[allow(dead_code)]
mod auth_paths;
#[allow(dead_code)]
mod billing_paths;
#[allow(dead_code)]
mod catalog_paths;
#[allow(dead_code)]
mod commerce_paths;
#[allow(dead_code)]
mod gateway_paths;
#[allow(dead_code)]
mod marketing_budget_paths;
#[allow(dead_code)]
mod marketing_campaign_paths;
#[allow(dead_code)]
mod marketing_code_paths;
#[allow(dead_code)]
mod marketing_runtime_paths;
#[allow(dead_code)]
mod marketing_template_paths;
#[allow(dead_code)]
mod system_paths;
#[allow(dead_code)]
mod tenant_paths;
#[allow(dead_code)]
mod user_paths;

fn admin_openapi() -> utoipa::openapi::OpenApi {
    AdminApiDoc::openapi()
}

async fn admin_openapi_handler() -> Json<utoipa::openapi::OpenApi> {
    Json(admin_openapi())
}

async fn admin_docs_index_handler() -> Html<String> {
    Html(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>SDKWORK Admin API</title>
    <style>
      :root {
        color-scheme: light dark;
        font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      }

      body {
        margin: 0;
        background: #f5f7fb;
        color: #101828;
      }

      .shell {
        display: grid;
        min-height: 100vh;
        grid-template-rows: auto 1fr;
      }

      .hero {
        padding: 20px 24px 16px;
        border-bottom: 1px solid rgba(15, 23, 42, 0.08);
        background: rgba(255, 255, 255, 0.96);
      }

      .eyebrow {
        margin: 0 0 8px;
        font-size: 12px;
        font-weight: 700;
        letter-spacing: 0.12em;
        text-transform: uppercase;
        color: #475467;
      }

      h1 {
        margin: 0 0 8px;
        font-size: 28px;
        line-height: 1.1;
      }

      p {
        margin: 0;
        font-size: 14px;
        line-height: 1.6;
        color: #475467;
      }

      code {
        padding: 2px 6px;
        border-radius: 999px;
        background: rgba(15, 23, 42, 0.06);
        font-size: 12px;
      }

      iframe {
        width: 100%;
        height: 100%;
        border: 0;
        background: white;
      }

      @media (prefers-color-scheme: dark) {
        body {
          background: #09090b;
          color: #fafafa;
        }

        .hero {
          background: rgba(24, 24, 27, 0.96);
          border-bottom-color: rgba(255, 255, 255, 0.08);
        }

        .eyebrow,
        p {
          color: #a1a1aa;
        }

        code {
          background: rgba(255, 255, 255, 0.08);
        }
      }
    </style>
  </head>
  <body>
    <main class="shell">
      <section class="hero">
        <p class="eyebrow">OpenAPI 3.1</p>
        <h1>SDKWORK Admin API</h1>
        <p>Interactive documentation is backed by the live schema endpoint <code>/admin/openapi.json</code>.</p>
      </section>
      <iframe src="/admin/docs/ui/" title="SDKWORK Admin API"></iframe>
    </main>
  </body>
</html>"#
            .to_string(),
    )
}

pub(crate) fn admin_docs_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/admin/openapi.json", get(admin_openapi_handler))
        .route("/admin/docs", get(admin_docs_index_handler))
        .merge(
            SwaggerUi::new("/admin/docs/ui/").config(SwaggerUiConfig::new([
                SwaggerUiUrl::with_primary("SDKWORK Admin API", "/admin/openapi.json", true),
            ])),
        )
}
