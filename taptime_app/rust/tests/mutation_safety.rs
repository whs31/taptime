use taptime_android_sdk::*;
#[tokio::test]
async fn missing_token_prevents_remote_mutation() {
    let sdk = TapTimeSdk::new("https://api.time.whs31.com".into()).unwrap();
    assert!(
        sdk.check_in(EventInput {
            date: "2026-11-14".into(),
            local_seconds: 9 * 3600
        })
        .await
        .is_err()
    );
    assert!(
        sdk.check_out(EventInput {
            date: "2026-11-14".into(),
            local_seconds: 17 * 3600
        })
        .await
        .is_err()
    );
}

#[test]
fn malformed_mutation_response_is_unknown_and_never_retryable() {
    let error = SdkError::api(taptime_client::ApiError::Wire("truncated frame"), true);
    let SdkError::Failure {
        retryable,
        mutation_outcome_unknown,
        ..
    } = error;
    assert!(!retryable);
    assert!(mutation_outcome_unknown);
}
