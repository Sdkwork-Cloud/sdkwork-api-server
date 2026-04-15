use super::error::MarketingServiceError;
use sdkwork_api_domain_marketing::{
    CouponRedemptionRecord, CouponRedemptionStatus, CouponReservationRecord,
    CouponReservationStatus,
};

#[allow(clippy::too_many_arguments)]
pub fn confirm_coupon_redemption(
    reservation: &CouponReservationRecord,
    coupon_redemption_id: impl Into<String>,
    coupon_code_id: impl Into<String>,
    coupon_template_id: impl Into<String>,
    budget_consumed_minor: u64,
    subsidy_amount_minor: u64,
    order_id: Option<String>,
    payment_event_id: Option<String>,
    now_ms: u64,
) -> Result<(CouponReservationRecord, CouponRedemptionRecord), MarketingServiceError> {
    if reservation.reservation_status != CouponReservationStatus::Reserved {
        return Err(MarketingServiceError::invalid_state(
            "reservation is not in reserved state",
        ));
    }
    if !reservation.is_active_at(now_ms) {
        return Err(MarketingServiceError::invalid_state(
            "reservation is no longer active",
        ));
    }
    if budget_consumed_minor > reservation.budget_reserved_minor {
        return Err(MarketingServiceError::invalid_state(
            "budget consumption exceeds reserved coupon budget",
        ));
    }

    let confirmed_reservation = reservation
        .clone()
        .with_status(CouponReservationStatus::Confirmed)
        .with_updated_at_ms(now_ms);
    let redemption = CouponRedemptionRecord::new(
        coupon_redemption_id,
        confirmed_reservation.coupon_reservation_id.clone(),
        coupon_code_id,
        coupon_template_id,
        now_ms,
    )
    .with_status(CouponRedemptionStatus::Redeemed)
    .with_budget_consumed_minor(budget_consumed_minor)
    .with_subsidy_amount_minor(subsidy_amount_minor)
    .with_order_id(order_id)
    .with_payment_event_id(payment_event_id)
    .with_updated_at_ms(now_ms);

    Ok((confirmed_reservation, redemption))
}
