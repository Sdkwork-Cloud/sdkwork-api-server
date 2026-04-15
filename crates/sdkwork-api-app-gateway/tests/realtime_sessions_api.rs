use sdkwork_api_app_gateway::create_realtime_session;

#[test]
fn local_realtime_session_fallback_returns_ephemeral_session() {
    let session = create_realtime_session("tenant-1", "project-1", "gpt-4o-realtime-preview")
        .expect("local fallback should synthesize a realtime session");

    assert!(session.id.starts_with("sess_local_"));
    assert_eq!(session.object, "realtime.session");
    assert_eq!(session.model, "gpt-4o-realtime-preview");
    let client_secret = session
        .client_secret
        .expect("local fallback should expose an ephemeral client secret");
    assert!(client_secret.value.starts_with("rtcs_local_"));
    assert!(client_secret.expires_at > 0);
}
