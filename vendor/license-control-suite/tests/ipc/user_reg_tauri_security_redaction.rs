use license_control_suite::modules::user_reg::auth_licensing_core::{AccessToken, DeviceKeyPair, DevicePublicKey, LicenseKey};
use license_control_suite::modules::user_reg::auth_licensing_tauri::AuthCommandError;

#[test]
fn domain_formatting_redacts_secret_values() {
    let license = LicenseKey::new("RAW-LICENSE-SECRET").unwrap();
    let token = AccessToken::new("RAW-TOKEN-SECRET").unwrap();
    let keypair =
        DeviceKeyPair::new(DevicePublicKey::new("public").unwrap(), "RAW-PRIVATE-KEY").unwrap();

    assert!(!format!("{license:?}").contains("RAW-LICENSE-SECRET"));
    assert!(!format!("{license}").contains("RAW-LICENSE-SECRET"));
    assert!(!format!("{token:?}").contains("RAW-TOKEN-SECRET"));
    assert!(!format!("{token}").contains("RAW-TOKEN-SECRET"));
    assert!(!format!("{keypair:?}").contains("RAW-PRIVATE-KEY"));
}

#[test]
fn command_errors_do_not_include_secret_values() {
    let error = AuthCommandError {
        code: "worker_unreachable".into(),
        message: "worker is unreachable".into(),
    };

    let json = serde_json::to_string(&error).unwrap();

    assert!(!json.contains("RAW-LICENSE-SECRET"));
    assert!(!json.contains("RAW-TOKEN-SECRET"));
}

#[test]
fn baseline_secret_redaction_behavior_is_preserved() {
    let license = LicenseKey::new("RAW-LICENSE-SECRET").unwrap();
    let token = AccessToken::new("RAW-TOKEN-SECRET").unwrap();
    let keypair =
        DeviceKeyPair::new(DevicePublicKey::new("public").unwrap(), "RAW-PRIVATE-KEY").unwrap();
    let error = AuthCommandError {
        code: "worker_unreachable".into(),
        message: "worker is unreachable".into(),
    };

    let formatted = [
        format!("{license:?}"),
        format!("{license}"),
        format!("{token:?}"),
        format!("{token}"),
        format!("{keypair:?}"),
        serde_json::to_string(&error).unwrap(),
    ];

    for rendered in formatted {
        assert!(!rendered.contains("RAW-LICENSE-SECRET"));
        assert!(!rendered.contains("RAW-TOKEN-SECRET"));
        assert!(!rendered.contains("RAW-PRIVATE-KEY"));
    }
}
