use sdkwork_api_app_billing::{book_usage_cost, check_quota};

#[test]
fn booking_usage_creates_ledger_entry() {
    assert!(check_quota("project-1", 100).unwrap());
    let ledger = book_usage_cost("project-1", 100, 0.25).unwrap();
    assert_eq!(ledger.project_id, "project-1");
}
