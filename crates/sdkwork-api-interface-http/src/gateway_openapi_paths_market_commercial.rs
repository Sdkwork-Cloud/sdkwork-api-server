use super::*;

#[utoipa::path(
    get,
    path = "/market/products",
    tag = "market",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Visible API products.", body = GatewayMarketProductsResponse),
        (status = 401, description = "Missing or invalid gateway API key.", body = GatewayApiErrorResponse),
        (status = 500, description = "Gateway failed to load API products.", body = GatewayApiErrorResponse)
    )
)]
pub(crate) async fn market_products() {}

#[utoipa::path(
    get,
    path = "/market/offers",
    tag = "market",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Visible product offers.", body = GatewayMarketOffersResponse),
        (status = 401, description = "Missing or invalid gateway API key.", body = GatewayApiErrorResponse),
        (status = 500, description = "Gateway failed to load product offers.", body = GatewayApiErrorResponse)
    )
)]
pub(crate) async fn market_offers() {}

#[utoipa::path(
    post,
    path = "/market/quotes",
    tag = "market",
    request_body = PortalCommerceQuoteRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Visible product quote.", body = PortalCommerceQuote),
        (status = 400, description = "Invalid quote payload.", body = GatewayApiErrorResponse),
        (status = 401, description = "Missing or invalid gateway API key.", body = GatewayApiErrorResponse),
        (status = 404, description = "Requested quote target was not found.", body = GatewayApiErrorResponse),
        (status = 500, description = "Gateway failed to preview the quote.", body = GatewayApiErrorResponse)
    )
)]
pub(crate) async fn market_quotes() {}

#[utoipa::path(
    post,
    path = "/marketing/coupons/validate",
    tag = "marketing",
    request_body = GatewayCouponValidationRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Coupon validation decision.", body = GatewayCouponValidationResponse),
        (status = 400, description = "Invalid coupon validation payload.", body = GatewayApiErrorResponse),
        (status = 401, description = "Missing or invalid gateway API key.", body = GatewayApiErrorResponse),
        (status = 404, description = "Requested coupon code was not found.", body = GatewayApiErrorResponse),
        (status = 429, description = "Coupon validation is rate limited.", body = GatewayApiErrorResponse),
        (status = 500, description = "Gateway failed to validate the coupon.", body = GatewayApiErrorResponse)
    )
)]
pub(crate) async fn marketing_coupon_validate() {}

#[utoipa::path(
    post,
    path = "/marketing/coupons/reserve",
    tag = "marketing",
    request_body = GatewayCouponReservationRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 201, description = "Reserved coupon redemption.", body = GatewayCouponReservationResponse),
        (status = 400, description = "Invalid coupon reservation payload.", body = GatewayApiErrorResponse),
        (status = 401, description = "Missing or invalid gateway API key.", body = GatewayApiErrorResponse),
        (status = 404, description = "Requested coupon code was not found.", body = GatewayApiErrorResponse),
        (status = 409, description = "Coupon reservation was rejected or conflicted.", body = GatewayApiErrorResponse),
        (status = 429, description = "Coupon reservation is rate limited.", body = GatewayApiErrorResponse),
        (status = 500, description = "Gateway failed to reserve the coupon.", body = GatewayApiErrorResponse)
    )
)]
pub(crate) async fn marketing_coupon_reserve() {}

#[utoipa::path(
    post,
    path = "/marketing/coupons/confirm",
    tag = "marketing",
    request_body = GatewayCouponRedemptionConfirmRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Confirmed coupon redemption.", body = GatewayCouponRedemptionConfirmResponse),
        (status = 400, description = "Invalid coupon confirmation payload.", body = GatewayApiErrorResponse),
        (status = 401, description = "Missing or invalid gateway API key.", body = GatewayApiErrorResponse),
        (status = 404, description = "Requested reservation or coupon code was not found.", body = GatewayApiErrorResponse),
        (status = 409, description = "Coupon confirmation conflicted with current runtime state.", body = GatewayApiErrorResponse),
        (status = 429, description = "Coupon confirmation is rate limited.", body = GatewayApiErrorResponse),
        (status = 500, description = "Gateway failed to confirm the coupon redemption.", body = GatewayApiErrorResponse)
    )
)]
pub(crate) async fn marketing_coupon_confirm() {}

#[utoipa::path(
    post,
    path = "/marketing/coupons/rollback",
    tag = "marketing",
    request_body = GatewayCouponRedemptionRollbackRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Completed coupon rollback.", body = GatewayCouponRedemptionRollbackResponse),
        (status = 400, description = "Invalid coupon rollback payload.", body = GatewayApiErrorResponse),
        (status = 401, description = "Missing or invalid gateway API key.", body = GatewayApiErrorResponse),
        (status = 404, description = "Requested redemption or coupon code was not found.", body = GatewayApiErrorResponse),
        (status = 409, description = "Coupon rollback conflicted with current runtime state.", body = GatewayApiErrorResponse),
        (status = 429, description = "Coupon rollback is rate limited.", body = GatewayApiErrorResponse),
        (status = 500, description = "Gateway failed to rollback the coupon redemption.", body = GatewayApiErrorResponse)
    )
)]
pub(crate) async fn marketing_coupon_rollback() {}

#[utoipa::path(
    get,
    path = "/commercial/account",
    tag = "commercial",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Resolved commercial account and balance.", body = GatewayCommercialAccountResponse),
        (status = 401, description = "Missing or invalid gateway API key.", body = GatewayApiErrorResponse),
        (status = 404, description = "Commercial account is not provisioned.", body = GatewayApiErrorResponse),
        (status = 501, description = "Commercial account runtime is unavailable for the current store.", body = GatewayApiErrorResponse),
        (status = 500, description = "Gateway failed to load the commercial account.", body = GatewayApiErrorResponse)
    )
)]
pub(crate) async fn commercial_account() {}

#[utoipa::path(
    get,
    path = "/commercial/account/benefit-lots",
    tag = "commercial",
    security(("bearerAuth" = [])),
    params(
        ("after_lot_id" = Option<u64>, Query, description = "Cursor lot id. Returns benefit lots strictly after this lot id for the current account."),
        ("limit" = Option<usize>, Query, description = "Maximum number of benefit lots to return. Defaults to 100 and is capped at 200.")
    ),
    responses(
        (status = 200, description = "Visible commercial account benefit lots.", body = GatewayCommercialBenefitLotsResponse),
        (status = 401, description = "Missing or invalid gateway API key.", body = GatewayApiErrorResponse),
        (status = 404, description = "Commercial account is not provisioned.", body = GatewayApiErrorResponse),
        (status = 501, description = "Commercial account runtime is unavailable for the current store.", body = GatewayApiErrorResponse),
        (status = 500, description = "Gateway failed to load commercial account benefit lots.", body = GatewayApiErrorResponse)
    )
)]
pub(crate) async fn commercial_account_benefit_lots() {}

