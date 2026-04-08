use super::*;

#[async_trait]
pub trait MarketingStore: AdminStore {
    async fn insert_coupon_template_record(
        &self,
        _record: &CouponTemplateRecord,
    ) -> Result<CouponTemplateRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_template_record",
        ))
    }

    async fn list_coupon_template_records(&self) -> Result<Vec<CouponTemplateRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_template_records",
        ))
    }

    async fn find_coupon_template_record(
        &self,
        _coupon_template_id: &str,
    ) -> Result<Option<CouponTemplateRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "find_coupon_template_record",
        ))
    }

    async fn find_coupon_template_record_by_template_key(
        &self,
        template_key: &str,
    ) -> Result<Option<CouponTemplateRecord>> {
        Ok(AdminStore::list_coupon_template_records(self)
            .await?
            .into_iter()
            .find(|record| record.template_key == template_key))
    }

    async fn insert_marketing_campaign_record(
        &self,
        _record: &MarketingCampaignRecord,
    ) -> Result<MarketingCampaignRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_marketing_campaign_record",
        ))
    }

    async fn list_marketing_campaign_records(&self) -> Result<Vec<MarketingCampaignRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_marketing_campaign_records",
        ))
    }

    async fn list_marketing_campaign_records_for_template(
        &self,
        coupon_template_id: &str,
    ) -> Result<Vec<MarketingCampaignRecord>> {
        Ok(AdminStore::list_marketing_campaign_records(self)
            .await?
            .into_iter()
            .filter(|record| record.coupon_template_id == coupon_template_id)
            .collect())
    }

    async fn insert_campaign_budget_record(
        &self,
        _record: &CampaignBudgetRecord,
    ) -> Result<CampaignBudgetRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_campaign_budget_record",
        ))
    }

    async fn list_campaign_budget_records(&self) -> Result<Vec<CampaignBudgetRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_campaign_budget_records",
        ))
    }

    async fn list_campaign_budget_records_for_campaign(
        &self,
        marketing_campaign_id: &str,
    ) -> Result<Vec<CampaignBudgetRecord>> {
        Ok(AdminStore::list_campaign_budget_records(self)
            .await?
            .into_iter()
            .filter(|record| record.marketing_campaign_id == marketing_campaign_id)
            .collect())
    }

    async fn insert_coupon_code_record(
        &self,
        _record: &CouponCodeRecord,
    ) -> Result<CouponCodeRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_code_record",
        ))
    }

    async fn list_coupon_code_records(&self) -> Result<Vec<CouponCodeRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_code_records",
        ))
    }

    async fn find_coupon_code_record(
        &self,
        _coupon_code_id: &str,
    ) -> Result<Option<CouponCodeRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "find_coupon_code_record",
        ))
    }

    async fn find_coupon_code_record_by_value(
        &self,
        code_value: &str,
    ) -> Result<Option<CouponCodeRecord>> {
        Ok(AdminStore::list_coupon_code_records(self)
            .await?
            .into_iter()
            .find(|record| record.code_value == code_value))
    }

    async fn list_redeemable_coupon_code_records_at(
        &self,
        now_ms: u64,
    ) -> Result<Vec<CouponCodeRecord>> {
        Ok(AdminStore::list_coupon_code_records(self)
            .await?
            .into_iter()
            .filter(|record| record.is_redeemable_at(now_ms))
            .collect())
    }

    async fn insert_coupon_reservation_record(
        &self,
        _record: &CouponReservationRecord,
    ) -> Result<CouponReservationRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_reservation_record",
        ))
    }

    async fn list_coupon_reservation_records(&self) -> Result<Vec<CouponReservationRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_reservation_records",
        ))
    }

    async fn find_coupon_reservation_record(
        &self,
        _coupon_reservation_id: &str,
    ) -> Result<Option<CouponReservationRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "find_coupon_reservation_record",
        ))
    }

    async fn list_active_coupon_reservation_records_at(
        &self,
        now_ms: u64,
    ) -> Result<Vec<CouponReservationRecord>> {
        Ok(AdminStore::list_coupon_reservation_records(self)
            .await?
            .into_iter()
            .filter(|record| record.is_active_at(now_ms))
            .collect())
    }

    async fn insert_coupon_redemption_record(
        &self,
        _record: &CouponRedemptionRecord,
    ) -> Result<CouponRedemptionRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_redemption_record",
        ))
    }

    async fn list_coupon_redemption_records(&self) -> Result<Vec<CouponRedemptionRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_redemption_records",
        ))
    }

    async fn find_coupon_redemption_record(
        &self,
        _coupon_redemption_id: &str,
    ) -> Result<Option<CouponRedemptionRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "find_coupon_redemption_record",
        ))
    }

    async fn insert_coupon_rollback_record(
        &self,
        _record: &CouponRollbackRecord,
    ) -> Result<CouponRollbackRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_rollback_record",
        ))
    }

    async fn list_coupon_rollback_records(&self) -> Result<Vec<CouponRollbackRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_rollback_records",
        ))
    }

    async fn insert_marketing_outbox_event_record(
        &self,
        _record: &MarketingOutboxEventRecord,
    ) -> Result<MarketingOutboxEventRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_marketing_outbox_event_record",
        ))
    }

    async fn list_marketing_outbox_event_records(&self) -> Result<Vec<MarketingOutboxEventRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_marketing_outbox_event_records",
        ))
    }
}

pub type MarketingKernelTransactionFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

#[async_trait]
pub trait MarketingKernelTransaction: Send {
    async fn upsert_coupon_template_record(
        &mut self,
        record: &CouponTemplateRecord,
    ) -> Result<CouponTemplateRecord>;

    async fn upsert_marketing_campaign_record(
        &mut self,
        record: &MarketingCampaignRecord,
    ) -> Result<MarketingCampaignRecord>;

    async fn find_coupon_reservation_record(
        &mut self,
        coupon_reservation_id: &str,
    ) -> Result<Option<CouponReservationRecord>>;

    async fn find_coupon_code_record(
        &mut self,
        coupon_code_id: &str,
    ) -> Result<Option<CouponCodeRecord>>;

    async fn find_campaign_budget_record(
        &mut self,
        campaign_budget_id: &str,
    ) -> Result<Option<CampaignBudgetRecord>>;

    async fn find_coupon_redemption_record(
        &mut self,
        coupon_redemption_id: &str,
    ) -> Result<Option<CouponRedemptionRecord>>;

    async fn find_coupon_rollback_record(
        &mut self,
        coupon_rollback_id: &str,
    ) -> Result<Option<CouponRollbackRecord>>;

    async fn list_marketing_campaign_records_for_template(
        &mut self,
        coupon_template_id: &str,
    ) -> Result<Vec<MarketingCampaignRecord>>;

    async fn list_campaign_budget_records_for_campaign(
        &mut self,
        marketing_campaign_id: &str,
    ) -> Result<Vec<CampaignBudgetRecord>>;

    async fn upsert_coupon_reservation_record(
        &mut self,
        record: &CouponReservationRecord,
    ) -> Result<CouponReservationRecord>;

    async fn upsert_coupon_code_record(
        &mut self,
        record: &CouponCodeRecord,
    ) -> Result<CouponCodeRecord>;

    async fn upsert_campaign_budget_record(
        &mut self,
        record: &CampaignBudgetRecord,
    ) -> Result<CampaignBudgetRecord>;

    async fn upsert_coupon_redemption_record(
        &mut self,
        record: &CouponRedemptionRecord,
    ) -> Result<CouponRedemptionRecord>;

    async fn upsert_coupon_rollback_record(
        &mut self,
        record: &CouponRollbackRecord,
    ) -> Result<CouponRollbackRecord>;

    async fn upsert_marketing_outbox_event_record(
        &mut self,
        record: &MarketingOutboxEventRecord,
    ) -> Result<MarketingOutboxEventRecord>;
}

pub trait MarketingKernelTransactionExecutor: MarketingStore {
    fn with_marketing_kernel_transaction<'a, T, F>(
        &'a self,
        operation: F,
    ) -> MarketingKernelTransactionFuture<'a, T>
    where
        T: Send + 'a,
        F: for<'tx> FnOnce(
                &'tx mut dyn MarketingKernelTransaction,
            ) -> MarketingKernelTransactionFuture<'tx, T>
            + Send
            + 'a;
}

pub async fn execute_atomic_coupon_reservation<E>(
    store: &E,
    command: &AtomicCouponReservationCommand,
) -> Result<AtomicCouponReservationResult>
where
    E: MarketingKernelTransactionExecutor + ?Sized,
{
    let command = command.clone();
    store
        .with_marketing_kernel_transaction(move |tx| {
            Box::pin(async move {
                if let Some(existing_reservation) = tx
                    .find_coupon_reservation_record(&command.reservation.coupon_reservation_id)
                    .await?
                {
                    if existing_reservation != command.reservation {
                        return Err(marketing_kernel_conflicting_replay(
                            "coupon reservation",
                            &command.reservation.coupon_reservation_id,
                        ));
                    }
                    let budget =
                        load_campaign_budget_snapshot(tx, &command.next_budget.campaign_budget_id)
                            .await?;
                    let code =
                        load_coupon_code_snapshot(tx, &command.next_code.coupon_code_id).await?;
                    ensure_marketing_record_matches(
                        "campaign budget",
                        &command.next_budget.campaign_budget_id,
                        Some(budget.clone()),
                        &command.next_budget,
                    )?;
                    ensure_marketing_record_matches(
                        "coupon code",
                        &command.next_code.coupon_code_id,
                        Some(code.clone()),
                        &command.next_code,
                    )?;
                    return Ok(AtomicCouponReservationResult {
                        budget,
                        code,
                        reservation: existing_reservation,
                        created: false,
                    });
                }

                if let Some(template) = command.template_to_persist.as_ref() {
                    tx.upsert_coupon_template_record(template).await?;
                }
                if let Some(campaign) = command.campaign_to_persist.as_ref() {
                    tx.upsert_marketing_campaign_record(campaign).await?;
                }
                if command.template_to_persist.is_some() || command.campaign_to_persist.is_some() {
                    if tx
                        .find_campaign_budget_record(&command.expected_budget.campaign_budget_id)
                        .await?
                        .is_none()
                    {
                        tx.upsert_campaign_budget_record(&command.expected_budget)
                            .await?;
                    }
                    if tx
                        .find_coupon_code_record(&command.expected_code.coupon_code_id)
                        .await?
                        .is_none()
                    {
                        tx.upsert_coupon_code_record(&command.expected_code).await?;
                    }
                }

                ensure_marketing_record_matches(
                    "campaign budget",
                    &command.expected_budget.campaign_budget_id,
                    tx.find_campaign_budget_record(&command.expected_budget.campaign_budget_id)
                        .await?,
                    &command.expected_budget,
                )?;
                ensure_marketing_record_matches(
                    "coupon code",
                    &command.expected_code.coupon_code_id,
                    tx.find_coupon_code_record(&command.expected_code.coupon_code_id)
                        .await?,
                    &command.expected_code,
                )?;

                let budget = tx
                    .upsert_campaign_budget_record(&command.next_budget)
                    .await?;
                let code = tx.upsert_coupon_code_record(&command.next_code).await?;
                let reservation = tx
                    .upsert_coupon_reservation_record(&command.reservation)
                    .await?;
                Ok(AtomicCouponReservationResult {
                    budget,
                    code,
                    reservation,
                    created: true,
                })
            })
        })
        .await
}

pub async fn execute_atomic_coupon_confirmation<E>(
    store: &E,
    command: &AtomicCouponConfirmationCommand,
) -> Result<AtomicCouponConfirmationResult>
where
    E: MarketingKernelTransactionExecutor + ?Sized,
{
    let command = command.clone();
    store
        .with_marketing_kernel_transaction(move |tx| {
            Box::pin(async move {
                if let Some(existing_redemption) = tx
                    .find_coupon_redemption_record(&command.redemption.coupon_redemption_id)
                    .await?
                {
                    if existing_redemption != command.redemption {
                        return Err(marketing_kernel_conflicting_replay(
                            "coupon redemption",
                            &command.redemption.coupon_redemption_id,
                        ));
                    }
                    let budget =
                        load_campaign_budget_snapshot(tx, &command.next_budget.campaign_budget_id)
                            .await?;
                    let code =
                        load_coupon_code_snapshot(tx, &command.next_code.coupon_code_id).await?;
                    let reservation = load_coupon_reservation_snapshot(
                        tx,
                        &command.next_reservation.coupon_reservation_id,
                    )
                    .await?;
                    ensure_marketing_record_matches(
                        "campaign budget",
                        &command.next_budget.campaign_budget_id,
                        Some(budget.clone()),
                        &command.next_budget,
                    )?;
                    ensure_marketing_record_matches(
                        "coupon code",
                        &command.next_code.coupon_code_id,
                        Some(code.clone()),
                        &command.next_code,
                    )?;
                    ensure_marketing_record_matches(
                        "coupon reservation",
                        &command.next_reservation.coupon_reservation_id,
                        Some(reservation.clone()),
                        &command.next_reservation,
                    )?;
                    return Ok(AtomicCouponConfirmationResult {
                        budget,
                        code,
                        reservation,
                        redemption: existing_redemption,
                        created: false,
                    });
                }

                ensure_marketing_record_matches(
                    "campaign budget",
                    &command.expected_budget.campaign_budget_id,
                    tx.find_campaign_budget_record(&command.expected_budget.campaign_budget_id)
                        .await?,
                    &command.expected_budget,
                )?;
                ensure_marketing_record_matches(
                    "coupon code",
                    &command.expected_code.coupon_code_id,
                    tx.find_coupon_code_record(&command.expected_code.coupon_code_id)
                        .await?,
                    &command.expected_code,
                )?;
                ensure_marketing_record_matches(
                    "coupon reservation",
                    &command.expected_reservation.coupon_reservation_id,
                    tx.find_coupon_reservation_record(
                        &command.expected_reservation.coupon_reservation_id,
                    )
                    .await?,
                    &command.expected_reservation,
                )?;

                let budget = tx
                    .upsert_campaign_budget_record(&command.next_budget)
                    .await?;
                let code = tx.upsert_coupon_code_record(&command.next_code).await?;
                let reservation = tx
                    .upsert_coupon_reservation_record(&command.next_reservation)
                    .await?;
                let redemption = tx
                    .upsert_coupon_redemption_record(&command.redemption)
                    .await?;
                Ok(AtomicCouponConfirmationResult {
                    budget,
                    code,
                    reservation,
                    redemption,
                    created: true,
                })
            })
        })
        .await
}

pub async fn execute_atomic_coupon_release<E>(
    store: &E,
    command: &AtomicCouponReleaseCommand,
) -> Result<AtomicCouponReleaseResult>
where
    E: MarketingKernelTransactionExecutor + ?Sized,
{
    let command = command.clone();
    store
        .with_marketing_kernel_transaction(move |tx| {
            Box::pin(async move {
                let current_reservation = load_coupon_reservation_snapshot(
                    tx,
                    &command.expected_reservation.coupon_reservation_id,
                )
                .await?;
                let budget =
                    load_campaign_budget_snapshot(tx, &command.expected_budget.campaign_budget_id)
                        .await?;
                let code =
                    load_coupon_code_snapshot(tx, &command.expected_code.coupon_code_id).await?;

                if current_reservation == command.next_reservation
                    && budget == command.next_budget
                    && code == command.next_code
                {
                    return Ok(AtomicCouponReleaseResult {
                        budget,
                        code,
                        reservation: current_reservation,
                        created: false,
                    });
                }

                ensure_marketing_record_matches(
                    "coupon reservation",
                    &command.expected_reservation.coupon_reservation_id,
                    Some(current_reservation),
                    &command.expected_reservation,
                )?;
                ensure_marketing_record_matches(
                    "campaign budget",
                    &command.expected_budget.campaign_budget_id,
                    Some(budget),
                    &command.expected_budget,
                )?;
                ensure_marketing_record_matches(
                    "coupon code",
                    &command.expected_code.coupon_code_id,
                    Some(code),
                    &command.expected_code,
                )?;

                let budget = tx
                    .upsert_campaign_budget_record(&command.next_budget)
                    .await?;
                let code = tx.upsert_coupon_code_record(&command.next_code).await?;
                let reservation = tx
                    .upsert_coupon_reservation_record(&command.next_reservation)
                    .await?;
                Ok(AtomicCouponReleaseResult {
                    budget,
                    code,
                    reservation,
                    created: true,
                })
            })
        })
        .await
}

pub async fn execute_atomic_coupon_rollback<E>(
    store: &E,
    command: &AtomicCouponRollbackCommand,
) -> Result<AtomicCouponRollbackResult>
where
    E: MarketingKernelTransactionExecutor + ?Sized,
{
    let command = command.clone();
    store
        .with_marketing_kernel_transaction(move |tx| {
            Box::pin(async move {
                if let Some(existing_rollback) = tx
                    .find_coupon_rollback_record(&command.rollback.coupon_rollback_id)
                    .await?
                {
                    if is_retryable_failed_coupon_rollback(&existing_rollback, &command.rollback) {
                        ensure_marketing_record_matches(
                            "campaign budget",
                            &command.expected_budget.campaign_budget_id,
                            tx.find_campaign_budget_record(
                                &command.expected_budget.campaign_budget_id,
                            )
                            .await?,
                            &command.expected_budget,
                        )?;
                        ensure_marketing_record_matches(
                            "coupon code",
                            &command.expected_code.coupon_code_id,
                            tx.find_coupon_code_record(&command.expected_code.coupon_code_id)
                                .await?,
                            &command.expected_code,
                        )?;
                        ensure_marketing_record_matches(
                            "coupon redemption",
                            &command.expected_redemption.coupon_redemption_id,
                            tx.find_coupon_redemption_record(
                                &command.expected_redemption.coupon_redemption_id,
                            )
                            .await?,
                            &command.expected_redemption,
                        )?;

                        let budget = tx
                            .upsert_campaign_budget_record(&command.next_budget)
                            .await?;
                        let code = tx.upsert_coupon_code_record(&command.next_code).await?;
                        let redemption = tx
                            .upsert_coupon_redemption_record(&command.next_redemption)
                            .await?;
                        let mut retry_rollback = command.rollback.clone();
                        retry_rollback.created_at_ms = existing_rollback.created_at_ms;
                        let rollback = tx.upsert_coupon_rollback_record(&retry_rollback).await?;
                        return Ok(AtomicCouponRollbackResult {
                            budget,
                            code,
                            redemption,
                            rollback,
                            created: false,
                        });
                    }
                    if existing_rollback != command.rollback {
                        return Err(marketing_kernel_conflicting_replay(
                            "coupon rollback",
                            &command.rollback.coupon_rollback_id,
                        ));
                    }
                    let budget =
                        load_campaign_budget_snapshot(tx, &command.next_budget.campaign_budget_id)
                            .await?;
                    let code =
                        load_coupon_code_snapshot(tx, &command.next_code.coupon_code_id).await?;
                    let redemption = load_coupon_redemption_snapshot(
                        tx,
                        &command.next_redemption.coupon_redemption_id,
                    )
                    .await?;
                    ensure_marketing_record_matches(
                        "campaign budget",
                        &command.next_budget.campaign_budget_id,
                        Some(budget.clone()),
                        &command.next_budget,
                    )?;
                    ensure_marketing_record_matches(
                        "coupon code",
                        &command.next_code.coupon_code_id,
                        Some(code.clone()),
                        &command.next_code,
                    )?;
                    ensure_marketing_record_matches(
                        "coupon redemption",
                        &command.next_redemption.coupon_redemption_id,
                        Some(redemption.clone()),
                        &command.next_redemption,
                    )?;
                    return Ok(AtomicCouponRollbackResult {
                        budget,
                        code,
                        redemption,
                        rollback: existing_rollback,
                        created: false,
                    });
                }

                ensure_marketing_record_matches(
                    "campaign budget",
                    &command.expected_budget.campaign_budget_id,
                    tx.find_campaign_budget_record(&command.expected_budget.campaign_budget_id)
                        .await?,
                    &command.expected_budget,
                )?;
                ensure_marketing_record_matches(
                    "coupon code",
                    &command.expected_code.coupon_code_id,
                    tx.find_coupon_code_record(&command.expected_code.coupon_code_id)
                        .await?,
                    &command.expected_code,
                )?;
                ensure_marketing_record_matches(
                    "coupon redemption",
                    &command.expected_redemption.coupon_redemption_id,
                    tx.find_coupon_redemption_record(
                        &command.expected_redemption.coupon_redemption_id,
                    )
                    .await?,
                    &command.expected_redemption,
                )?;

                let budget = tx
                    .upsert_campaign_budget_record(&command.next_budget)
                    .await?;
                let code = tx.upsert_coupon_code_record(&command.next_code).await?;
                let redemption = tx
                    .upsert_coupon_redemption_record(&command.next_redemption)
                    .await?;
                let rollback = tx.upsert_coupon_rollback_record(&command.rollback).await?;
                Ok(AtomicCouponRollbackResult {
                    budget,
                    code,
                    redemption,
                    rollback,
                    created: true,
                })
            })
        })
        .await
}

pub async fn execute_atomic_coupon_rollback_compensation<E>(
    store: &E,
    command: &AtomicCouponRollbackCompensationCommand,
) -> Result<AtomicCouponRollbackCompensationResult>
where
    E: MarketingKernelTransactionExecutor + ?Sized,
{
    let command = command.clone();
    store
        .with_marketing_kernel_transaction(move |tx| {
            Box::pin(async move {
                let existing_rollback = load_coupon_rollback_snapshot(
                    tx,
                    &command.expected_rollback.coupon_rollback_id,
                )
                .await?;
                if existing_rollback == command.next_rollback {
                    let budget =
                        load_campaign_budget_snapshot(tx, &command.next_budget.campaign_budget_id)
                            .await?;
                    let code =
                        load_coupon_code_snapshot(tx, &command.next_code.coupon_code_id).await?;
                    let redemption = load_coupon_redemption_snapshot(
                        tx,
                        &command.next_redemption.coupon_redemption_id,
                    )
                    .await?;
                    ensure_marketing_record_matches(
                        "campaign budget",
                        &command.next_budget.campaign_budget_id,
                        Some(budget.clone()),
                        &command.next_budget,
                    )?;
                    ensure_marketing_record_matches(
                        "coupon code",
                        &command.next_code.coupon_code_id,
                        Some(code.clone()),
                        &command.next_code,
                    )?;
                    ensure_marketing_record_matches(
                        "coupon redemption",
                        &command.next_redemption.coupon_redemption_id,
                        Some(redemption.clone()),
                        &command.next_redemption,
                    )?;
                    return Ok(AtomicCouponRollbackCompensationResult {
                        budget,
                        code,
                        redemption,
                        rollback: existing_rollback,
                        created: false,
                    });
                }

                ensure_marketing_record_matches(
                    "coupon rollback",
                    &command.expected_rollback.coupon_rollback_id,
                    Some(existing_rollback),
                    &command.expected_rollback,
                )?;
                ensure_marketing_record_matches(
                    "campaign budget",
                    &command.expected_budget.campaign_budget_id,
                    tx.find_campaign_budget_record(&command.expected_budget.campaign_budget_id)
                        .await?,
                    &command.expected_budget,
                )?;
                ensure_marketing_record_matches(
                    "coupon code",
                    &command.expected_code.coupon_code_id,
                    tx.find_coupon_code_record(&command.expected_code.coupon_code_id)
                        .await?,
                    &command.expected_code,
                )?;
                ensure_marketing_record_matches(
                    "coupon redemption",
                    &command.expected_redemption.coupon_redemption_id,
                    tx.find_coupon_redemption_record(
                        &command.expected_redemption.coupon_redemption_id,
                    )
                    .await?,
                    &command.expected_redemption,
                )?;

                let budget = tx
                    .upsert_campaign_budget_record(&command.next_budget)
                    .await?;
                let code = tx.upsert_coupon_code_record(&command.next_code).await?;
                let redemption = tx
                    .upsert_coupon_redemption_record(&command.next_redemption)
                    .await?;
                let rollback = tx
                    .upsert_coupon_rollback_record(&command.next_rollback)
                    .await?;
                Ok(AtomicCouponRollbackCompensationResult {
                    budget,
                    code,
                    redemption,
                    rollback,
                    created: true,
                })
            })
        })
        .await
}

async fn load_campaign_budget_snapshot(
    tx: &mut dyn MarketingKernelTransaction,
    campaign_budget_id: &str,
) -> Result<CampaignBudgetRecord> {
    tx.find_campaign_budget_record(campaign_budget_id)
        .await?
        .ok_or_else(|| marketing_kernel_missing_record("campaign budget", campaign_budget_id))
}

async fn load_coupon_code_snapshot(
    tx: &mut dyn MarketingKernelTransaction,
    coupon_code_id: &str,
) -> Result<CouponCodeRecord> {
    tx.find_coupon_code_record(coupon_code_id)
        .await?
        .ok_or_else(|| marketing_kernel_missing_record("coupon code", coupon_code_id))
}

async fn load_coupon_reservation_snapshot(
    tx: &mut dyn MarketingKernelTransaction,
    coupon_reservation_id: &str,
) -> Result<CouponReservationRecord> {
    tx.find_coupon_reservation_record(coupon_reservation_id)
        .await?
        .ok_or_else(|| marketing_kernel_missing_record("coupon reservation", coupon_reservation_id))
}

async fn load_coupon_redemption_snapshot(
    tx: &mut dyn MarketingKernelTransaction,
    coupon_redemption_id: &str,
) -> Result<CouponRedemptionRecord> {
    tx.find_coupon_redemption_record(coupon_redemption_id)
        .await?
        .ok_or_else(|| marketing_kernel_missing_record("coupon redemption", coupon_redemption_id))
}

async fn load_coupon_rollback_snapshot(
    tx: &mut dyn MarketingKernelTransaction,
    coupon_rollback_id: &str,
) -> Result<CouponRollbackRecord> {
    tx.find_coupon_rollback_record(coupon_rollback_id)
        .await?
        .ok_or_else(|| marketing_kernel_missing_record("coupon rollback", coupon_rollback_id))
}

fn is_retryable_failed_coupon_rollback(
    existing_rollback: &CouponRollbackRecord,
    requested_rollback: &CouponRollbackRecord,
) -> bool {
    existing_rollback.coupon_rollback_id == requested_rollback.coupon_rollback_id
        && existing_rollback.coupon_redemption_id == requested_rollback.coupon_redemption_id
        && existing_rollback.rollback_type == requested_rollback.rollback_type
        && existing_rollback.rollback_status == CouponRollbackStatus::Failed
        && existing_rollback.restored_budget_minor == requested_rollback.restored_budget_minor
        && existing_rollback.restored_inventory_count == requested_rollback.restored_inventory_count
}

fn ensure_marketing_record_matches<T: PartialEq>(
    record_type: &str,
    record_id: &str,
    actual: Option<T>,
    expected: &T,
) -> Result<()> {
    match actual {
        Some(actual) if actual == *expected => Ok(()),
        Some(_) => Err(marketing_kernel_snapshot_changed(record_type, record_id)),
        None => Err(marketing_kernel_missing_record(record_type, record_id)),
    }
}

fn marketing_kernel_snapshot_changed(record_type: &str, record_id: &str) -> anyhow::Error {
    anyhow::anyhow!("{record_type} {record_id} changed concurrently")
}

fn marketing_kernel_missing_record(record_type: &str, record_id: &str) -> anyhow::Error {
    anyhow::anyhow!("{record_type} {record_id} is missing")
}

fn marketing_kernel_conflicting_replay(record_type: &str, record_id: &str) -> anyhow::Error {
    anyhow::anyhow!("{record_type} {record_id} already exists with different state")
}
