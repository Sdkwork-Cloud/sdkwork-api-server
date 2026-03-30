use anyhow::{anyhow, Result};
use sdkwork_api_domain_coupon::CouponCampaign;
use sdkwork_api_storage_core::AdminStore;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn persist_coupon(
    store: &dyn AdminStore,
    coupon: &CouponCampaign,
) -> Result<CouponCampaign> {
    validate_coupon(coupon)?;
    let next = if coupon.created_at_ms == 0 {
        coupon.clone().with_created_at_ms(current_time_ms()?)
    } else {
        coupon.clone()
    };
    store.insert_coupon(&next).await
}

pub async fn list_coupons(store: &dyn AdminStore) -> Result<Vec<CouponCampaign>> {
    store.list_coupons().await
}

pub async fn list_active_coupons(store: &dyn AdminStore) -> Result<Vec<CouponCampaign>> {
    store.list_active_coupons().await
}

pub async fn delete_coupon(store: &dyn AdminStore, coupon_id: &str) -> Result<bool> {
    let coupon_id = coupon_id.trim();
    if coupon_id.is_empty() {
        return Err(anyhow!("coupon id is required"));
    }
    store.delete_coupon(coupon_id).await
}

fn validate_coupon(coupon: &CouponCampaign) -> Result<()> {
    if coupon.id.trim().is_empty() {
        return Err(anyhow!("coupon id is required"));
    }
    if coupon.code.trim().is_empty() {
        return Err(anyhow!("coupon code is required"));
    }
    if coupon.discount_label.trim().is_empty() {
        return Err(anyhow!("discount_label is required"));
    }
    if coupon.audience.trim().is_empty() {
        return Err(anyhow!("audience is required"));
    }
    if coupon.note.trim().is_empty() {
        return Err(anyhow!("note is required"));
    }
    if coupon.expires_on.trim().is_empty() {
        return Err(anyhow!("expires_on is required"));
    }
    Ok(())
}

fn current_time_ms() -> Result<u64> {
    Ok(u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("system clock error"))?
            .as_millis(),
    )?)
}
