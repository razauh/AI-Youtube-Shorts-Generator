use std::fs;
use std::path::Path;

#[cfg(feature = "core")]
#[path = "../../examples/core-only.rs"]
mod core_only_example;

#[cfg(feature = "desktop-tauri")]
#[path = "../../examples/desktop-tauri-host.rs"]
mod desktop_tauri_host_example;

#[cfg(feature = "core")]
#[path = "../../examples/fake-dependency-injection.rs"]
mod fake_dependency_injection_example;

#[cfg(feature = "desktop-tauri")]
#[path = "../../examples/host-command-composition.rs"]
mod host_command_composition_example;

#[cfg(feature = "core")]
#[path = "../../examples/admin-desktop-console.rs"]
mod admin_desktop_console_example;

#[cfg(feature = "core")]
#[tokio::test]
async fn core_only_usage_example_compiles() {
    let state = core_only_example::run_core_only_activation_flow()
        .await
        .expect("core-only example should succeed");

    assert!(matches!(
        state,
        license_control_suite::core::SessionState::Licensed { .. }
    ));
}

#[cfg(feature = "desktop-tauri")]
#[test]
fn desktop_tauri_usage_example_compiles() {
    let names = desktop_tauri_host_example::command_surface_names();

    assert_eq!(names, license_control_suite::app_command_names());
    let _ = desktop_tauri_host_example::build_desktop_handler::<tauri::Wry>();
}

#[cfg(feature = "core")]
#[tokio::test]
async fn fake_dependency_injection_example_compiles() {
    let state = fake_dependency_injection_example::run_fake_injection_activation_flow()
        .await
        .expect("fake injection example should succeed");

    assert!(matches!(
        state,
        license_control_suite::core::SessionState::Licensed { .. }
    ));
}

#[cfg(feature = "desktop-tauri")]
#[test]
fn host_command_composition_example_compiles() {
    assert_eq!(
        host_command_composition_example::composed_command_names(),
        license_control_suite::desktop::tauri::command_names(),
    );

    let _ = host_command_composition_example::build_host_owned_handler::<tauri::Wry>();
    let _ = host_command_composition_example::build_crate_owned_handler::<tauri::Wry>();
}

#[cfg(feature = "core")]
#[test]
fn admin_desktop_console_onboarding_path_is_documented() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = fs::read_to_string(root.join("README.md")).expect("README.md should exist");
    let note = fs::read_to_string(root.join("docs/consumer_onboarding.md"))
        .expect("consumer onboarding guide should exist");
    let session = admin_desktop_console_example::run_admin_login_flow()
        .expect("admin desktop example should succeed");

    assert_eq!(session.operator_id, "operator-1");
    assert!(readme.contains("Admin desktop console onboarding"));
    assert!(note.contains("desktop admin console"));
    assert!(note.contains("not a web dashboard"));
}

#[test]
fn unsupported_and_deferred_capabilities_are_explicitly_listed() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = fs::read_to_string(root.join("README.md")).expect("README.md should exist");
    let note = fs::read_to_string(root.join("docs/consumer_onboarding.md"))
        .expect("consumer onboarding guide should exist");

    for capability in [
        "web apps",
        "mobile apps",
        "Cloudflare runtime",
        "Gumroad integration",
        "payment flows",
        "hosted SaaS backend",
    ] {
        assert!(readme.contains(capability), "README should mention {capability}");
        assert!(
            note.contains(capability),
            "consumer onboarding note should mention {capability}"
        );
    }
}

#[test]
fn tauri_onboarding_examples_use_curated_handler_helper_instead_of_direct_macro_calls() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let desktop_host = fs::read_to_string(root.join("examples/desktop-tauri-host.rs"))
        .expect("desktop tauri host example should exist");
    let host_composition =
        fs::read_to_string(root.join("examples/host-command-composition.rs"))
            .expect("host command composition example should exist");

    assert!(desktop_host.contains("auth_command_handler::<R>()"));
    assert!(host_composition.contains("auth_command_handler::<R>()"));
    assert!(!desktop_host.contains("tauri::generate_handler!["));
    assert!(!host_composition.contains("tauri::generate_handler!["));
}
