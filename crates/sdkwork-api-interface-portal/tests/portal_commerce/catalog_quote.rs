use super::*;

#[tokio::test]
async fn portal_commerce_catalog_exposes_plans_packs_and_active_coupons() {
    let pool = memory_pool().await;
    sqlx::query(
        "INSERT INTO ai_coupon_campaigns (id, code, discount_label, audience, remaining, active, note, expires_on, created_at_ms)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("coupon_spring_launch")
    .bind("SPRING20")
    .bind("20% launch discount")
    .bind("new_signup")
    .bind(120_i64)
    .bind(1_i64)
    .bind("Spring launch campaign")
    .bind("2026-05-31")
    .bind(1_710_000_001_i64)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO ai_coupon_campaigns (id, code, discount_label, audience, remaining, active, note, expires_on, created_at_ms)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("coupon_inactive")
    .bind("INACTIVE10")
    .bind("10% inactive discount")
    .bind("internal")
    .bind(40_i64)
    .bind(0_i64)
    .bind("Inactive campaign")
    .bind("2026-05-31")
    .bind(1_710_000_002_i64)
    .execute(&pool)
    .await
    .unwrap();

    let app = portal_lab_app(pool);
    let token = portal_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/catalog")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["plans"].as_array().unwrap().len(), 3);
    assert_eq!(json["packs"].as_array().unwrap().len(), 3);
    assert_eq!(json["products"].as_array().unwrap().len(), 7);
    assert_eq!(json["offers"].as_array().unwrap().len(), 7);
    assert!(json["products"].as_array().unwrap().iter().any(|product| {
        product["product_kind"] == "subscription_plan"
            && product["target_id"] == "growth"
            && product["display_name"] == "Growth"
    }));
    assert!(json["products"].as_array().unwrap().iter().any(|product| {
        product["product_kind"] == "custom_recharge" && product["target_id"] == "custom_recharge"
    }));
    assert!(json["offers"].as_array().unwrap().iter().any(|offer| {
        offer["quote_target_kind"] == "recharge_pack"
            && offer["quote_target_id"] == "pack-100k"
            && offer["quote_kind"] == "product_purchase"
            && offer["publication_id"] == "publication:portal_catalog:offer:recharge_pack:pack-100k"
            && offer["publication_kind"] == "portal_catalog"
            && offer["publication_status"] == "published"
            && offer["publication_revision_id"]
                == "publication_revision:portal_catalog:offer:recharge_pack:pack-100k:v1"
            && offer["publication_version"] == 1
            && offer["publication_source_kind"] == "catalog_seed"
            && offer["pricing_plan_id"] == "pricing_plan:recharge_pack:pack-100k"
            && offer["pricing_plan_version"] == 1
            && offer["pricing_rate_id"]
                == "pricing_rate:recharge_pack:pack-100k:credit.prepaid_pack"
            && offer["pricing_metric_code"] == "credit.prepaid_pack"
    }));
    assert!(json["coupons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|coupon| coupon["code"] == "SPRING20"));
    assert!(json["coupons"]
        .as_array()
        .unwrap()
        .iter()
        .all(|coupon| coupon["code"] != "INACTIVE10"));
}

#[tokio::test]
async fn portal_commerce_catalog_prefers_active_pricing_governance_from_account_kernel() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_pricing_plan(&store, 9103, "recharge_pack:pack-100k", 3, "active").await;
    seed_pricing_plan(&store, 9104, "recharge_pack:pack-100k", 4, "planned").await;

    let app = portal_lab_app(pool);
    let token = portal_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/catalog")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    let governed_offer = json["offers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|offer| {
            offer["quote_target_kind"] == "recharge_pack" && offer["quote_target_id"] == "pack-100k"
        })
        .expect("pack offer");
    assert_eq!(governed_offer["pricing_plan_version"], 3);
    assert_eq!(governed_offer["publication_status"], "published");
    assert_eq!(
        governed_offer["publication_revision_id"],
        "publication_revision:portal_catalog:offer:recharge_pack:pack-100k:v3"
    );
    assert_eq!(governed_offer["publication_version"], 3);
    assert_eq!(governed_offer["publication_source_kind"], "pricing_plan");
    assert_eq!(
        governed_offer["publication_effective_from_ms"],
        1_710_000_000_000_u64
    );
}

#[tokio::test]
async fn portal_commerce_catalog_exposes_server_managed_recharge_options_and_custom_policy() {
    let pool = memory_pool().await;
    let app = portal_lab_app(pool);
    let token = portal_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/catalog")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["recharge_options"].as_array().unwrap().len(), 4);
    assert_eq!(json["recharge_options"][0]["amount_cents"], 1000);
    assert_eq!(json["recharge_options"][0]["amount_label"], "$10.00");
    assert_eq!(json["recharge_options"][0]["granted_units"], 25000);
    assert_eq!(
        json["recharge_options"][1]["effective_ratio_label"],
        "2,800 units / $1"
    );
    assert_eq!(json["custom_recharge_policy"]["enabled"], true);
    assert_eq!(json["custom_recharge_policy"]["min_amount_cents"], 1000);
    assert_eq!(json["custom_recharge_policy"]["step_amount_cents"], 500);
    assert_eq!(
        json["custom_recharge_policy"]["suggested_amount_cents"],
        5000
    );
    assert_eq!(
        json["custom_recharge_policy"]["rules"][1]["effective_ratio_label"],
        "2,800 units / $1"
    );
    assert!(json["offers"].as_array().unwrap().iter().any(|offer| {
        offer["quote_target_kind"] == "custom_recharge"
            && offer["quote_target_id"] == "custom_recharge"
    }));
}

#[tokio::test]
async fn portal_commerce_catalog_reclaims_expired_marketing_coupon_reservations_inline() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_marketing_catalog_coupon(&store).await;

    let app = portal_lab_app(pool);
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    let original_code = store
        .find_coupon_code_record_by_value("LAUNCH20")
        .await
        .unwrap()
        .unwrap();
    let reserved_code = original_code
        .clone()
        .with_status(CouponCodeStatus::Reserved)
        .with_updated_at_ms(10);
    store
        .insert_coupon_code_record(&reserved_code)
        .await
        .unwrap();

    let original_budget = store
        .list_campaign_budget_records()
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let reserved_budget = original_budget
        .clone()
        .with_reserved_budget_minor(1_200)
        .with_updated_at_ms(10);
    store
        .insert_campaign_budget_record(&reserved_budget)
        .await
        .unwrap();

    let expired_reservation = CouponReservationRecord::new(
        "reservation_catalog_expired",
        reserved_code.coupon_code_id.clone(),
        MarketingSubjectScope::Project,
        project_id,
        1,
    )
    .with_status(CouponReservationStatus::Reserved)
    .with_budget_reserved_minor(1_200)
    .with_created_at_ms(0)
    .with_updated_at_ms(10);
    store
        .insert_coupon_reservation_record(&expired_reservation)
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/catalog")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert!(json["coupons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|coupon| coupon["code"] == "LAUNCH20"));

    let reservations = store.list_coupon_reservation_records().await.unwrap();
    assert_eq!(reservations.len(), 1);
    assert_eq!(
        reservations[0].reservation_status,
        CouponReservationStatus::Expired
    );

    let refreshed_code = store
        .find_coupon_code_record_by_value("LAUNCH20")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(refreshed_code.status, CouponCodeStatus::Available);

    let refreshed_budget = store
        .list_campaign_budget_records()
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(refreshed_budget.reserved_budget_minor, 0);
}

#[tokio::test]
async fn portal_commerce_catalog_requires_authentication() {
    let pool = memory_pool().await;
    let app = portal_lab_app(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/catalog")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn portal_commerce_quote_prices_recharge_and_coupon_redemption() {
    let pool = memory_pool().await;
    sqlx::query(
        "INSERT INTO ai_coupon_campaigns (id, code, discount_label, audience, remaining, active, note, expires_on, created_at_ms)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("coupon_spring_launch")
    .bind("SPRING20")
    .bind("20% launch discount")
    .bind("new_signup")
    .bind(120_i64)
    .bind(1_i64)
    .bind("Spring launch campaign")
    .bind("2026-05-31")
    .bind(1_710_000_001_i64)
    .execute(&pool)
    .await
    .unwrap();

    let app = portal_lab_app(pool);
    let token = portal_token(app.clone()).await;

    let recharge_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/quote")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\",\"coupon_code\":\"SPRING20\",\"current_remaining_units\":5000}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(recharge_response.status(), StatusCode::OK);
    let recharge_json = read_json(recharge_response).await;
    assert_eq!(recharge_json["target_kind"], "recharge_pack");
    assert_eq!(recharge_json["target_name"], "Boost 100k");
    assert_eq!(recharge_json["list_price_label"], "$40.00");
    assert_eq!(recharge_json["payable_price_label"], "$32.00");
    assert_eq!(recharge_json["granted_units"], 100000);
    assert_eq!(recharge_json["projected_remaining_units"], 105000);
    assert_eq!(recharge_json["applied_coupon"]["code"], "SPRING20");
    assert_eq!(
        recharge_json["pricing_plan_id"],
        "pricing_plan:recharge_pack:pack-100k"
    );
    assert_eq!(recharge_json["pricing_plan_version"], 1);
    assert_eq!(
        recharge_json["pricing_rate_id"],
        "pricing_rate:recharge_pack:pack-100k:credit.prepaid_pack"
    );
    assert_eq!(recharge_json["pricing_metric_code"], "credit.prepaid_pack");
    assert_eq!(
        recharge_json["product_id"],
        "product:recharge_pack:pack-100k"
    );
    assert_eq!(recharge_json["offer_id"], "offer:recharge_pack:pack-100k");
    assert_eq!(
        recharge_json["publication_id"],
        "publication:portal_catalog:offer:recharge_pack:pack-100k"
    );
    assert_eq!(recharge_json["publication_kind"], "portal_catalog");
    assert_eq!(recharge_json["publication_status"], "published");
    assert_eq!(
        recharge_json["publication_revision_id"],
        "publication_revision:portal_catalog:offer:recharge_pack:pack-100k:v1"
    );
    assert_eq!(recharge_json["publication_version"], 1);
    assert_eq!(recharge_json["publication_source_kind"], "catalog_seed");

    let coupon_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/quote")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"coupon_redemption\",\"target_id\":\"WELCOME100\",\"current_remaining_units\":5000}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(coupon_response.status(), StatusCode::OK);
    let coupon_json = read_json(coupon_response).await;
    assert_eq!(coupon_json["target_kind"], "coupon_redemption");
    assert_eq!(coupon_json["target_name"], "WELCOME100");
    assert_eq!(coupon_json["payable_price_label"], "$0.00");
    assert_eq!(coupon_json["bonus_units"], 100);
    assert_eq!(coupon_json["projected_remaining_units"], 5100);
}

#[tokio::test]
async fn portal_commerce_quote_and_order_support_custom_recharge_from_server_policy() {
    let pool = memory_pool().await;
    let app = portal_lab_app(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(&project_id)
    .bind(240_i64)
    .bind(0.42_f64)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(&project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(&pool)
    .await
    .unwrap();

    let quote_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/quote")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"custom_recharge\",\"target_id\":\"custom\",\"custom_amount_cents\":5000,\"current_remaining_units\":260}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(quote_response.status(), StatusCode::OK);
    let quote_json = read_json(quote_response).await;
    assert_eq!(quote_json["target_kind"], "custom_recharge");
    assert_eq!(quote_json["target_id"], "custom-5000");
    assert_eq!(quote_json["target_name"], "Custom recharge");
    assert_eq!(quote_json["amount_cents"], 5000);
    assert_eq!(quote_json["list_price_label"], "$50.00");
    assert_eq!(quote_json["payable_price_label"], "$50.00");
    assert_eq!(quote_json["granted_units"], 140000);
    assert_eq!(quote_json["projected_remaining_units"], 140260);
    assert_eq!(quote_json["pricing_rule_label"], "Tiered custom recharge");
    assert_eq!(quote_json["effective_ratio_label"], "2,800 units / $1");
    assert_eq!(
        quote_json["pricing_plan_id"],
        "pricing_plan:custom_recharge:custom_recharge"
    );
    assert_eq!(quote_json["pricing_plan_version"], 1);
    assert_eq!(
        quote_json["pricing_rate_id"],
        "pricing_rate:custom_recharge:custom_recharge:credit.prepaid_custom"
    );
    assert_eq!(quote_json["pricing_metric_code"], "credit.prepaid_custom");
    assert_eq!(
        quote_json["product_id"],
        "product:custom_recharge:custom_recharge"
    );
    assert_eq!(
        quote_json["offer_id"],
        "offer:custom_recharge:custom_recharge"
    );
    assert_eq!(
        quote_json["publication_id"],
        "publication:portal_catalog:offer:custom_recharge:custom_recharge"
    );
    assert_eq!(quote_json["publication_kind"], "portal_catalog");
    assert_eq!(quote_json["publication_status"], "published");
    assert_eq!(
        quote_json["publication_revision_id"],
        "publication_revision:portal_catalog:offer:custom_recharge:custom_recharge:v1"
    );
    assert_eq!(quote_json["publication_version"], 1);
    assert_eq!(quote_json["publication_source_kind"], "catalog_seed");

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"custom_recharge\",\"target_id\":\"custom\",\"custom_amount_cents\":5000}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_json = read_json(create_response).await;
    let order_id = create_json["order_id"].as_str().unwrap().to_owned();
    assert_eq!(create_json["target_kind"], "custom_recharge");
    assert_eq!(create_json["target_id"], "custom-5000");
    assert_eq!(create_json["target_name"], "Custom recharge");
    assert_eq!(create_json["payable_price_label"], "$50.00");
    assert_eq!(create_json["granted_units"], 140000);
    assert_eq!(create_json["status"], "pending_payment");
    assert_eq!(
        create_json["pricing_plan_id"],
        "pricing_plan:custom_recharge:custom_recharge"
    );
    assert_eq!(create_json["pricing_plan_version"], 1);
    assert_eq!(
        create_json["publication_revision_id"],
        "publication_revision:portal_catalog:offer:custom_recharge:custom_recharge:v1"
    );
    assert_eq!(create_json["publication_version"], 1);
    assert_eq!(create_json["publication_source_kind"], "catalog_seed");

    let settle_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/portal/commerce/orders/{order_id}/settle"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(settle_response.status(), StatusCode::OK);
    let settle_json = read_json(settle_response).await;
    assert_eq!(settle_json["status"], "fulfilled");

    let billing_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(billing_response.status(), StatusCode::OK);
    let billing_json = read_json(billing_response).await;
    assert_eq!(billing_json["remaining_units"], 140260);
}
