use super::super::support::load_subject_reservations;
use super::super::types::MarketingSubjectSet;
use anyhow::Result;
use sdkwork_api_domain_marketing::{CouponRedemptionRecord, CouponRedemptionStatus};
use sdkwork_api_storage_core::AdminStore;

pub async fn list_coupon_redemptions_for_subjects(
    store: &dyn AdminStore,
    subjects: &MarketingSubjectSet,
    status: Option<CouponRedemptionStatus>,
) -> Result<Vec<CouponRedemptionRecord>> {
    let reservation_ids = load_subject_reservations(store, subjects)
        .await?
        .into_iter()
        .map(|reservation| reservation.coupon_reservation_id)
        .collect::<Vec<_>>();

    let mut redemptions = store
        .list_coupon_redemption_records_for_reservation_ids(&reservation_ids)
        .await?
        .into_iter()
        .filter(|redemption| {
            status.map_or(true, |expected| redemption.redemption_status == expected)
        })
        .collect::<Vec<_>>();

    redemptions.sort_by(|left, right| {
        right
            .redeemed_at_ms
            .cmp(&left.redeemed_at_ms)
            .then_with(|| right.coupon_redemption_id.cmp(&left.coupon_redemption_id))
    });
    Ok(redemptions)
}
