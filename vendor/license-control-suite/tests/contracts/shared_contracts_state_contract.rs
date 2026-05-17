use license_control_suite::modules::shared_contracts::state::{LicenseState, ResetRequestState};

#[test]
fn license_state_serializes_to_canonical_values() {
    let cases = [
        (LicenseState::Unbound, "\"UNBOUND\""),
        (LicenseState::BoundActive, "\"BOUND_ACTIVE\""),
        (LicenseState::ResetPending, "\"RESET_PENDING\""),
        (LicenseState::Suspended, "\"SUSPENDED\""),
        (LicenseState::Revoked, "\"REVOKED\""),
    ];

    for (state, expected) in cases {
        let got = serde_json::to_string(&state).unwrap();
        assert_eq!(got, expected);
    }
}

#[test]
fn reset_request_state_serializes_to_canonical_values() {
    let cases = [
        (ResetRequestState::Submitted, "\"SUBMITTED\""),
        (ResetRequestState::UnderReview, "\"UNDER_REVIEW\""),
        (ResetRequestState::Approved, "\"APPROVED\""),
        (ResetRequestState::Rejected, "\"REJECTED\""),
        (ResetRequestState::Expired, "\"EXPIRED\""),
        (ResetRequestState::Cancelled, "\"CANCELLED\""),
    ];

    for (state, expected) in cases {
        let got = serde_json::to_string(&state).unwrap();
        assert_eq!(got, expected);
    }
}

#[test]
fn unknown_states_fail_to_deserialize() {
    let bad_license = serde_json::from_str::<LicenseState>("\"NOPE\"");
    let bad_reset = serde_json::from_str::<ResetRequestState>("\"NOPE\"");

    assert!(bad_license.is_err());
    assert!(bad_reset.is_err());
}
