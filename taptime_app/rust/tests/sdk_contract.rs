use taptime_android_sdk::*;

#[test]
fn rejects_unsecured_non_loopback_endpoints() {
    assert!(TapTimeSdk::new("http://example.com".into()).is_err());
    assert!(TapTimeSdk::new("https://example.com/suspicious/path".into()).is_err());
}

#[test]
fn rejects_invalid_session_tokens() {
    let sdk = TapTimeSdk::new("https://api.time.whs31.com".into()).unwrap();
    assert!(sdk.set_session_token("token\nsecret".into()).is_err());
}
