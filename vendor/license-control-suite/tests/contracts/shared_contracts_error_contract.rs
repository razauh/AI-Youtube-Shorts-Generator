use license_control_suite::modules::shared_contracts::errors::{ApiError, ErrorCode};

#[test]
fn api_error_wire_shape_is_stable() {
    let err = ApiError::new(
        ErrorCode::LicenseInvalid,
        "Human-readable summary",
        false,
        "req_123",
    );

    let got = serde_json::to_value(&err).unwrap();
    let expected = serde_json::json!({
        "error": {
            "code": "LICENSE_INVALID",
            "message": "Human-readable summary",
            "retryable": false
        },
        "request_id": "req_123"
    });

    assert_eq!(got, expected);
}

#[test]
fn error_codes_are_canonical_strings() {
    let codes = [
        ErrorCode::LicenseInvalid,
        ErrorCode::LicenseRevoked,
        ErrorCode::LicenseSuspended,
        ErrorCode::LicenseAlreadyBound,
        ErrorCode::SessionReauthRequired,
        ErrorCode::ResetRequestNotFound,
        ErrorCode::ResetRequestConflict,
        ErrorCode::AdminAuthInvalid,
        ErrorCode::AdminForbidden,
    ];

    for code in codes {
        let s = serde_json::to_string(&code).unwrap();
        assert!(s.starts_with('"') && s.ends_with('"'));
    }
}
