use crate::contracts::SdkErrorCode;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum SdkError {
    #[error("{message}")]
    Failure {
        code: SdkErrorCode,
        message: String,
        retryable: bool,
        mutation_outcome_unknown: bool,
    },
}

impl SdkError {
    pub fn input(msg: impl Into<String>) -> Self {
        Self::Failure {
            code: SdkErrorCode::InvalidInput,
            message: msg.into(),
            retryable: false,
            mutation_outcome_unknown: false,
        }
    }
    pub fn unauthorized() -> Self {
        Self::Failure {
            code: SdkErrorCode::Unauthenticated,
            message: "Login required".into(),
            retryable: false,
            mutation_outcome_unknown: false,
        }
    }
    pub fn api(err: taptime_client::ApiError, mutation: bool) -> Self {
        use taptime_client::ApiError;
        let (code, retryable, unknown) = match &err {
            ApiError::Unauthenticated => (SdkErrorCode::Unauthenticated, false, false),
            ApiError::Grpc { code: 5, .. } => (SdkErrorCode::NotFound, false, false),
            ApiError::Grpc { code: 6, .. } => (SdkErrorCode::Conflict, false, false),
            ApiError::Grpc { code: 7, .. } => (SdkErrorCode::PermissionDenied, false, false),
            ApiError::Grpc { code: 16, .. } => (SdkErrorCode::Unauthenticated, false, false),
            ApiError::Network(e) if e.is_timeout() => (SdkErrorCode::Timeout, !mutation, mutation),
            ApiError::Network(_) => (SdkErrorCode::Unavailable, !mutation, mutation),
            ApiError::HttpStatus(429 | 502 | 503 | 504) => {
                (SdkErrorCode::Unavailable, !mutation, mutation)
            }
            ApiError::Wire(_) | ApiError::Missing(_) | ApiError::Protobuf(_) => {
                (SdkErrorCode::InvalidResponse, false, mutation)
            }
            _ => (SdkErrorCode::Internal, false, false),
        };
        let message = match code {
            SdkErrorCode::Unauthenticated => "Session expired".into(),
            SdkErrorCode::Timeout => "Request timed out; outcome may be unknown".into(),
            SdkErrorCode::Unavailable => "Connection unavailable".into(),
            _ => "TapTime request failed".into(),
        };
        Self::Failure {
            code,
            message,
            retryable,
            mutation_outcome_unknown: unknown,
        }
    }
}
