use super::campaign::load_marketing_campaign;
use super::coupon::{
    load_coupon_code, load_coupon_redemption, load_coupon_reservation, load_coupon_rollbacks,
    load_coupon_template,
};
use super::types::MarketingOrderEvidenceView;
use anyhow::Result;
use sdkwork_api_storage_core::AdminStore;

pub async fn load_marketing_order_evidence(
    store: &dyn AdminStore,
    coupon_reservation_id: Option<&str>,
    coupon_redemption_id: Option<&str>,
    applied_coupon_code: Option<&str>,
    marketing_campaign_id: Option<&str>,
) -> Result<MarketingOrderEvidenceView> {
    let coupon_reservation = load_coupon_reservation(store, coupon_reservation_id).await?;
    let coupon_redemption = load_coupon_redemption(store, coupon_redemption_id).await?;
    let coupon_rollbacks = load_coupon_rollbacks(store, coupon_redemption.as_ref()).await?;
    let coupon_code = load_coupon_code(
        store,
        coupon_reservation.as_ref(),
        coupon_redemption.as_ref(),
        applied_coupon_code,
    )
    .await?;
    let coupon_template =
        load_coupon_template(store, coupon_redemption.as_ref(), coupon_code.as_ref()).await?;
    let marketing_campaign = load_marketing_campaign(store, marketing_campaign_id).await?;

    Ok(MarketingOrderEvidenceView {
        coupon_reservation,
        coupon_redemption,
        coupon_rollbacks,
        coupon_code,
        coupon_template,
        marketing_campaign,
    })
}
