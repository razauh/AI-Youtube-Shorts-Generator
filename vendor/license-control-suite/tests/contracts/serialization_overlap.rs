use license_control_suite::modules::shared_contracts::dto::ActivateRequest as SharedActivateRequest;
use license_control_suite::modules::shared_contracts::state::LicenseState as SharedLicenseState;
use license_control_suite::modules::user_reg::auth_licensing_core::ActivationRequest as CoreActivationRequest;
use license_control_suite::modules::user_reg::auth_licensing_core::SessionState as CoreSessionState;

#[test]
fn shared_and_user_reg_types_have_explicit_separate_paths() {
    let shared_name = std::any::type_name::<SharedActivateRequest>();
    let core_name = std::any::type_name::<CoreActivationRequest>();
    assert_ne!(shared_name, core_name);
    assert!(shared_name.contains("shared_contracts"));
    assert!(core_name.contains("auth_licensing_core"));
}

#[test]
fn similarly_named_state_types_are_not_unified() {
    let shared_state_name = std::any::type_name::<SharedLicenseState>();
    let core_state_name = std::any::type_name::<CoreSessionState>();
    assert_ne!(shared_state_name, core_state_name);
    assert!(shared_state_name.contains("shared_contracts"));
    assert!(core_state_name.contains("auth_licensing_core"));
}
