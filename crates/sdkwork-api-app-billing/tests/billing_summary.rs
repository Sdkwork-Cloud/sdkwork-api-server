use sdkwork_api_app_billing::summarize_billing_snapshot;
use sdkwork_api_domain_billing::{LedgerEntry, QuotaPolicy};

#[test]
fn summarizes_billing_posture_and_quota_exhaustion_by_project() {
    let entries = vec![
        LedgerEntry::new("project-1", 40, 0.40),
        LedgerEntry::new("project-1", 70, 0.70),
        LedgerEntry::new("project-2", 10, 0.10),
    ];
    let policies = vec![
        QuotaPolicy::new("quota-project-1", "project-1", 100),
        QuotaPolicy::new("quota-project-2", "project-2", 500).with_enabled(false),
        QuotaPolicy::new("quota-project-3", "project-3", 200),
    ];

    let summary = summarize_billing_snapshot(&entries, &policies);

    assert_eq!(summary.total_entries, 3);
    assert_eq!(summary.project_count, 3);
    assert_eq!(summary.total_units, 120);
    assert!((summary.total_amount - 1.20).abs() < 1e-9);
    assert_eq!(summary.active_quota_policy_count, 2);
    assert_eq!(summary.exhausted_project_count, 1);

    assert_eq!(summary.projects.len(), 3);

    assert_eq!(summary.projects[0].project_id, "project-1");
    assert_eq!(summary.projects[0].entry_count, 2);
    assert_eq!(summary.projects[0].used_units, 110);
    assert!((summary.projects[0].booked_amount - 1.10).abs() < 1e-9);
    assert_eq!(
        summary.projects[0].quota_policy_id.as_deref(),
        Some("quota-project-1")
    );
    assert_eq!(summary.projects[0].quota_limit_units, Some(100));
    assert_eq!(summary.projects[0].remaining_units, Some(0));
    assert!(summary.projects[0].exhausted);

    assert_eq!(summary.projects[1].project_id, "project-3");
    assert_eq!(summary.projects[1].entry_count, 0);
    assert_eq!(summary.projects[1].used_units, 0);
    assert!((summary.projects[1].booked_amount - 0.0).abs() < 1e-9);
    assert_eq!(
        summary.projects[1].quota_policy_id.as_deref(),
        Some("quota-project-3")
    );
    assert_eq!(summary.projects[1].quota_limit_units, Some(200));
    assert_eq!(summary.projects[1].remaining_units, Some(200));
    assert!(!summary.projects[1].exhausted);

    assert_eq!(summary.projects[2].project_id, "project-2");
    assert_eq!(summary.projects[2].entry_count, 1);
    assert_eq!(summary.projects[2].used_units, 10);
    assert!((summary.projects[2].booked_amount - 0.10).abs() < 1e-9);
    assert_eq!(summary.projects[2].quota_policy_id, None);
    assert_eq!(summary.projects[2].quota_limit_units, None);
    assert_eq!(summary.projects[2].remaining_units, None);
    assert!(!summary.projects[2].exhausted);
}
