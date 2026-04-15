#![allow(clippy::await_holding_lock)]

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use std::sync::{Mutex, OnceLock};
use tower::ServiceExt;

fn http_exposure_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[tokio::test]
async fn openapi_routes_expose_portal_api_inventory_with_schema_components() {
    let _lock = http_exposure_env_lock().lock().unwrap();
    let app = sdkwork_api_interface_portal::portal_router();

    let openapi = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/portal/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(openapi.status(), StatusCode::OK);
    let bytes = to_bytes(openapi.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["openapi"], "3.1.0");
    assert_eq!(json["info"]["title"], "SDKWORK Portal API");
    assert!(json["paths"]["/portal/health"]["get"].is_object());
    assert!(json["paths"]["/portal/auth/login"]["post"].is_object());
    assert!(json["paths"]["/portal/workspace"]["get"].is_object());
    assert!(json["paths"]["/portal/marketing/coupon-validations"]["post"].is_object());
    assert!(json["paths"]["/portal/marketing/coupon-reservations"]["post"].is_object());
    assert!(json["paths"]["/portal/marketing/coupon-redemptions/confirm"]["post"].is_object());
    assert!(json["paths"]["/portal/marketing/coupon-redemptions/rollback"]["post"].is_object());
    assert!(json["paths"]["/portal/marketing/my-coupons"]["get"].is_object());
    assert!(json["paths"]["/portal/marketing/reward-history"]["get"].is_object());
    assert!(json["paths"]["/portal/marketing/redemptions"]["get"].is_object());
    assert!(json["paths"]["/portal/marketing/codes"]["get"].is_object());
    assert!(json["paths"]["/portal/commerce/catalog"]["get"].is_object());
    assert!(json["paths"]["/portal/commerce/orders/{order_id}"]["get"].is_object());
    assert!(json["paths"]["/portal/commerce/orders/{order_id}/payment-methods"]["get"].is_object());
    assert!(json["paths"]["/portal/commerce/order-center"]["get"].is_object());
    assert!(
        json["paths"]["/portal/commerce/payment-attempts/{payment_attempt_id}"]["get"].is_object()
    );
    assert!(json["paths"]["/portal/billing/account"]["get"].is_object());
    assert!(json["paths"]["/portal/billing/account-history"]["get"].is_object());
    assert!(json["paths"]["/portal/billing/account/balance"]["get"].is_object());
    assert!(json["paths"]["/portal/billing/account/benefit-lots"]["get"].is_object());
    assert!(json["paths"]["/portal/billing/account/holds"]["get"].is_object());
    assert!(json["paths"]["/portal/billing/account/request-settlements"]["get"].is_object());
    assert!(json["paths"]["/portal/billing/account/ledger"]["get"].is_object());
    assert!(json["paths"]["/portal/billing/pricing-plans"]["get"].is_object());
    assert!(json["paths"]["/portal/billing/pricing-rates"]["get"].is_object());
    assert!(json["paths"]["/portal/async-jobs"]["get"].is_object());
    assert!(json["paths"]["/portal/async-jobs/{job_id}/attempts"]["get"].is_object());
    assert!(json["paths"]["/portal/async-jobs/{job_id}/assets"]["get"].is_object());
    assert!(json["components"]["schemas"].is_object());
    assert!(json["components"]["schemas"]["ErrorResponse"].is_object());
    assert!(json["components"]["schemas"]["LoginRequest"].is_object());
    assert!(json["components"]["schemas"]["PortalAuthSession"].is_object());
    assert!(json["components"]["schemas"]["PortalWorkspaceSummary"].is_object());
    assert!(json["components"]["schemas"]["PortalCouponValidationRequest"].is_object());
    assert!(json["components"]["schemas"]["PortalCouponValidationResponse"].is_object());
    assert!(json["components"]["schemas"]["PortalCouponReservationRequest"].is_object());
    assert!(json["components"]["schemas"]["PortalCouponReservationResponse"].is_object());
    assert!(json["components"]["schemas"]["PortalCouponRedemptionConfirmRequest"].is_object());
    assert!(json["components"]["schemas"]["PortalCouponRedemptionConfirmResponse"].is_object());
    assert!(json["components"]["schemas"]["PortalCouponRedemptionRollbackRequest"].is_object());
    assert!(json["components"]["schemas"]["PortalCouponRedemptionRollbackResponse"].is_object());
    assert!(json["components"]["schemas"]["PortalCouponApplicabilitySummary"].is_object());
    assert!(json["components"]["schemas"]["PortalCouponEffectSummary"].is_object());
    assert!(json["components"]["schemas"]["PortalCouponOwnershipSummary"].is_object());
    assert!(json["components"]["schemas"]["PortalMarketingCodeItem"].is_object());
    assert!(json["components"]["schemas"]["PortalMarketingRewardHistoryItem"].is_object());
    assert!(json["components"]["schemas"]["PortalMarketingCodesResponse"].is_object());
    assert!(json["components"]["schemas"]["PortalMarketingRedemptionsResponse"].is_object());
    assert!(json["components"]["schemas"]["PortalCommerceCatalog"].is_object());
    assert!(json["components"]["schemas"]["PortalApiProduct"].is_object());
    assert!(json["components"]["schemas"]["PortalProductOffer"].is_object());
    assert!(json["components"]["schemas"]["PortalCommerceOrder"].is_object());
    assert!(json["components"]["schemas"]["PortalCommerceOrderCenterResponse"].is_object());
    assert!(json["components"]["schemas"]["PaymentMethodRecord"].is_object());
    assert!(json["components"]["schemas"]["CommercePaymentAttemptRecord"].is_object());
    assert!(json["components"]["schemas"]["PortalBillingAccountHistoryResponse"].is_object());
    assert!(json["components"]["schemas"]["PortalBillingAccountResponse"].is_object());
    assert!(json["components"]["schemas"]["AccountBalanceSnapshot"].is_object());
    assert!(json["components"]["schemas"]["AccountBenefitLotRecord"].is_object());
    assert!(json["components"]["schemas"]["AccountHoldRecord"].is_object());
    assert!(json["components"]["schemas"]["RequestSettlementRecord"].is_object());
    assert!(json["components"]["schemas"]["AccountLedgerHistoryEntry"].is_object());
    assert!(json["components"]["schemas"]["PricingPlanRecord"].is_object());
    assert!(json["components"]["schemas"]["PricingRateRecord"].is_object());
    assert!(json["components"]["schemas"]["AsyncJobRecord"].is_object());
    assert!(json["components"]["schemas"]["AsyncJobAttemptRecord"].is_object());
    assert!(json["components"]["schemas"]["AsyncJobAssetRecord"].is_object());

    let billing_history_schema =
        &json["components"]["schemas"]["PortalBillingAccountHistoryResponse"];
    assert_eq!(billing_history_schema["type"], "object");
    assert_eq!(
        billing_history_schema["properties"]["benefit_lots"]["type"],
        "array"
    );
    assert_eq!(
        billing_history_schema["properties"]["benefit_lots"]["items"]["$ref"],
        "#/components/schemas/AccountBenefitLotRecord"
    );
    assert_eq!(
        billing_history_schema["properties"]["holds"]["type"],
        "array"
    );
    assert_eq!(
        billing_history_schema["properties"]["holds"]["items"]["$ref"],
        "#/components/schemas/AccountHoldRecord"
    );
    assert_eq!(
        billing_history_schema["properties"]["request_settlements"]["type"],
        "array"
    );
    assert_eq!(
        billing_history_schema["properties"]["request_settlements"]["items"]["$ref"],
        "#/components/schemas/RequestSettlementRecord"
    );
    assert_eq!(
        billing_history_schema["properties"]["ledger"]["type"],
        "array"
    );
    assert_eq!(
        billing_history_schema["properties"]["ledger"]["items"]["$ref"],
        "#/components/schemas/AccountLedgerHistoryEntry"
    );
    assert_eq!(
        json["paths"]["/portal/auth/login"]["post"]["requestBody"]["content"]["application/json"]
            ["schema"]["$ref"],
        "#/components/schemas/LoginRequest"
    );
    assert_eq!(
        json["paths"]["/portal/auth/login"]["post"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["$ref"],
        "#/components/schemas/PortalAuthSession"
    );
    assert_eq!(
        json["paths"]["/portal/workspace"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["$ref"],
        "#/components/schemas/PortalWorkspaceSummary"
    );
    assert_eq!(
        json["paths"]["/portal/workspace"]["get"]["security"][0]["bearerAuth"],
        serde_json::json!([])
    );
    assert_eq!(
        json["paths"]["/portal/marketing/coupon-validations"]["post"]["requestBody"]["content"]
            ["application/json"]["schema"]["$ref"],
        "#/components/schemas/PortalCouponValidationRequest"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalCouponValidationRequest"]["properties"]["target_kind"]
            ["type"],
        "string"
    );
    assert_eq!(
        json["paths"]["/portal/marketing/coupon-validations"]["post"]["responses"]["200"]
            ["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/PortalCouponValidationResponse"
    );
    assert_eq!(
        json["paths"]["/portal/marketing/coupon-reservations"]["post"]["requestBody"]["content"]
            ["application/json"]["schema"]["$ref"],
        "#/components/schemas/PortalCouponReservationRequest"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalCouponReservationRequest"]["properties"]
            ["target_kind"]["type"],
        "string"
    );
    assert_eq!(
        json["paths"]["/portal/marketing/coupon-reservations"]["post"]["responses"]["200"]
            ["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/PortalCouponReservationResponse"
    );
    assert_eq!(
        json["paths"]["/portal/marketing/my-coupons"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["$ref"],
        "#/components/schemas/PortalMarketingCodesResponse"
    );
    assert_eq!(
        json["paths"]["/portal/marketing/reward-history"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["items"]["$ref"],
        "#/components/schemas/PortalMarketingRewardHistoryItem"
    );
    assert_eq!(
        json["paths"]["/portal/marketing/redemptions"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["$ref"],
        "#/components/schemas/PortalMarketingRedemptionsResponse"
    );
    assert_eq!(
        json["paths"]["/portal/marketing/codes"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["$ref"],
        "#/components/schemas/PortalMarketingCodesResponse"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalMarketingCodesResponse"]["properties"]["items"]
            ["items"]["$ref"],
        "#/components/schemas/PortalMarketingCodeItem"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalMarketingCodeItem"]["properties"]["template"]["$ref"],
        "#/components/schemas/CouponTemplateRecord"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalMarketingCodeItem"]["properties"]["campaign"]["$ref"],
        "#/components/schemas/MarketingCampaignRecord"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalMarketingCodeItem"]["properties"]["applicability"]
            ["$ref"],
        "#/components/schemas/PortalCouponApplicabilitySummary"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalMarketingCodeItem"]["properties"]["effect"]["$ref"],
        "#/components/schemas/PortalCouponEffectSummary"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalMarketingCodeItem"]["properties"]["ownership"]["$ref"],
        "#/components/schemas/PortalCouponOwnershipSummary"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalMarketingRewardHistoryItem"]["properties"]
            ["account_arrival"]["$ref"],
        "#/components/schemas/PortalCouponAccountArrivalSummary"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalCouponAccountArrivalSummary"]["properties"]
            ["benefit_lots"]["items"]["$ref"],
        "#/components/schemas/PortalCouponAccountArrivalLotItem"
    );
    assert_eq!(
        json["paths"]["/portal/commerce/catalog"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["$ref"],
        "#/components/schemas/PortalCommerceCatalog"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalCommerceCatalog"]["properties"]["products"]["items"]
            ["$ref"],
        "#/components/schemas/PortalApiProduct"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalCommerceCatalog"]["properties"]["offers"]["items"]
            ["$ref"],
        "#/components/schemas/PortalProductOffer"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalProductOffer"]["properties"]
            ["publication_revision_id"]["type"],
        "string"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalProductOffer"]["properties"]["publication_version"]
            ["type"],
        "integer"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalProductOffer"]["properties"]
            ["publication_source_kind"]["type"],
        "string"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalProductOffer"]["properties"]
            ["publication_effective_from_ms"]["type"][0],
        "integer"
    );
    assert_eq!(
        json["paths"]["/portal/commerce/orders/{order_id}"]["get"]["parameters"][0]["name"],
        "order_id"
    );
    assert_eq!(
        json["paths"]["/portal/commerce/orders/{order_id}"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["$ref"],
        "#/components/schemas/PortalCommerceOrder"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalCommerceOrder"]["properties"]["product_kind"]["type"],
        "string"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalCommerceOrder"]["properties"]["transaction_kind"]
            ["type"],
        "string"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalCommerceOrder"]["properties"]["product_id"]["type"],
        "string"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalCommerceOrder"]["properties"]["offer_id"]["type"],
        "string"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalCommerceOrder"]["properties"]["publication_id"]
            ["type"],
        "string"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalCommerceOrder"]["properties"]["publication_kind"]
            ["type"],
        "string"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalCommerceOrder"]["properties"]["publication_status"]
            ["type"],
        "string"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalCommerceOrder"]["properties"]
            ["publication_revision_id"]["type"],
        "string"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalCommerceOrder"]["properties"]["publication_version"]
            ["type"],
        "integer"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalCommerceOrder"]["properties"]
            ["publication_source_kind"]["type"],
        "string"
    );
    assert_eq!(
        json["components"]["schemas"]["PortalCommerceOrder"]["properties"]
            ["publication_effective_from_ms"]["type"][0],
        "integer"
    );
    assert_eq!(
        json["paths"]["/portal/commerce/orders/{order_id}/payment-methods"]["get"]["parameters"][0]
            ["name"],
        "order_id"
    );
    assert_eq!(
        json["paths"]["/portal/commerce/orders/{order_id}/payment-methods"]["get"]["responses"]
            ["200"]["content"]["application/json"]["schema"]["items"]["$ref"],
        "#/components/schemas/PaymentMethodRecord"
    );
    assert_eq!(
        json["paths"]["/portal/commerce/order-center"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["$ref"],
        "#/components/schemas/PortalCommerceOrderCenterResponse"
    );
    assert_eq!(
        json["paths"]["/portal/commerce/payment-attempts/{payment_attempt_id}"]["get"]
            ["parameters"][0]["name"],
        "payment_attempt_id"
    );
    assert_eq!(
        json["paths"]["/portal/commerce/payment-attempts/{payment_attempt_id}"]["get"]["responses"]
            ["200"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/CommercePaymentAttemptRecord"
    );
    assert_eq!(
        json["paths"]["/portal/billing/account"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["$ref"],
        "#/components/schemas/PortalBillingAccountResponse"
    );
    assert_eq!(
        json["paths"]["/portal/billing/account-history"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["$ref"],
        "#/components/schemas/PortalBillingAccountHistoryResponse"
    );
    assert_eq!(
        json["paths"]["/portal/billing/account/balance"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["$ref"],
        "#/components/schemas/AccountBalanceSnapshot"
    );
    assert_eq!(
        json["paths"]["/portal/billing/account/benefit-lots"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["type"],
        "array"
    );
    assert_eq!(
        json["paths"]["/portal/billing/account/benefit-lots"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["items"]["$ref"],
        "#/components/schemas/AccountBenefitLotRecord"
    );
    assert_eq!(
        json["paths"]["/portal/billing/account/holds"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["items"]["$ref"],
        "#/components/schemas/AccountHoldRecord"
    );
    assert_eq!(
        json["paths"]["/portal/billing/account/request-settlements"]["get"]["responses"]["200"]
            ["content"]["application/json"]["schema"]["items"]["$ref"],
        "#/components/schemas/RequestSettlementRecord"
    );
    assert_eq!(
        json["paths"]["/portal/billing/account/ledger"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["items"]["$ref"],
        "#/components/schemas/AccountLedgerHistoryEntry"
    );
    assert_eq!(
        json["paths"]["/portal/billing/pricing-plans"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["items"]["$ref"],
        "#/components/schemas/PricingPlanRecord"
    );
    assert_eq!(
        json["paths"]["/portal/billing/pricing-rates"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["items"]["$ref"],
        "#/components/schemas/PricingRateRecord"
    );
    assert_eq!(
        json["paths"]["/portal/async-jobs"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["items"]["$ref"],
        "#/components/schemas/AsyncJobRecord"
    );
    assert_eq!(
        json["paths"]["/portal/async-jobs/{job_id}/attempts"]["get"]["parameters"][0]["name"],
        "job_id"
    );
    assert_eq!(
        json["paths"]["/portal/async-jobs/{job_id}/attempts"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["items"]["$ref"],
        "#/components/schemas/AsyncJobAttemptRecord"
    );
    assert_eq!(
        json["paths"]["/portal/async-jobs/{job_id}/assets"]["get"]["parameters"][0]["name"],
        "job_id"
    );
    assert_eq!(
        json["paths"]["/portal/async-jobs/{job_id}/assets"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["items"]["$ref"],
        "#/components/schemas/AsyncJobAssetRecord"
    );
    assert_eq!(
        json["paths"]["/portal/marketing/coupon-validations"]["post"]["security"][0]["bearerAuth"],
        serde_json::json!([])
    );
    assert_eq!(
        json["paths"]["/portal/billing/account"]["get"]["security"][0]["bearerAuth"],
        serde_json::json!([])
    );
    assert_eq!(
        json["paths"]["/portal/async-jobs"]["get"]["security"][0]["bearerAuth"],
        serde_json::json!([])
    );
    assert!(
        json["paths"]["/portal/auth/login"]["post"]["security"].is_null()
            || json["paths"]["/portal/auth/login"]["post"]["security"]
                .as_array()
                .is_some_and(Vec::is_empty)
    );

    let docs = app
        .oneshot(
            Request::builder()
                .uri("/portal/docs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(docs.status(), StatusCode::OK);
    let bytes = to_bytes(docs.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("SDKWORK Portal API"));
    assert!(html.contains("/portal/openapi.json"));
}

#[test]
fn try_portal_router_returns_error_for_invalid_http_exposure_env() {
    let _lock = http_exposure_env_lock().lock().unwrap();
    let key = "SDKWORK_BROWSER_ALLOWED_ORIGINS";
    let previous = std::env::var(key).ok();
    std::env::set_var(key, ";;;");

    let result = sdkwork_api_interface_portal::try_portal_router();

    match previous {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }

    let error = result.expect_err("invalid env should return an error");
    assert!(error
        .to_string()
        .contains("invalid list value for SDKWORK_BROWSER_ALLOWED_ORIGINS"));
}

#[tokio::test]
async fn try_portal_router_with_pool_returns_error_for_invalid_http_exposure_env() {
    let _lock = http_exposure_env_lock().lock().unwrap();
    let key = "SDKWORK_BROWSER_ALLOWED_ORIGINS";
    let previous = std::env::var(key).ok();
    std::env::set_var(key, ";;;");
    let pool = sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap();

    let result = sdkwork_api_interface_portal::try_portal_router_with_pool(pool);

    match previous {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }

    let error = result.expect_err("invalid env should return an error");
    assert!(error
        .to_string()
        .contains("invalid list value for SDKWORK_BROWSER_ALLOWED_ORIGINS"));
}
