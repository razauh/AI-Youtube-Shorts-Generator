use license_control_suite::modules::shared_contracts::versioning::{CompatibilityInfo, SemverChange};

#[test]
fn compatibility_info_reports_expected_range() {
    let info = CompatibilityInfo::current();
    assert_eq!(info.contract_version, "1.0.0");
    assert_eq!(info.supported_min, "1.0.0");
    assert_eq!(info.supported_max_exclusive, "2.0.0");
}

#[test]
fn semver_policy_flags_breaking_minor_change() {
    let result = CompatibilityInfo::is_change_allowed(SemverChange::Minor, false);
    assert!(result.is_ok());

    let breaking_minor = CompatibilityInfo::is_change_allowed(SemverChange::Minor, true);
    assert!(breaking_minor.is_err());
}
