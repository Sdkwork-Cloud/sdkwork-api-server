use sdkwork_api_domain_billing::LedgerEntry;

#[test]
fn ledger_entry_tracks_units() {
    let entry = LedgerEntry::new("project-1", 100, 0.25);
    assert_eq!(entry.units, 100);
}
